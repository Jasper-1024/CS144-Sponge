use std::process;

use cs144_rust::{
    tcp_helpers::{
        tcp_header::{TCPHeader, TCPHeaderTrait},
        tcp_segment::{TCPSegment, TCPSegmentTrait},
    },
    util::{
        buffer::Buffer,
        parser::{NetParser, ParseError},
        util::{hexdump, InternetChecksum},
    },
};
use pcap::Capture;
use rand::{thread_rng, Rng};

const NREPS: usize = 32;

fn inet_cksum(data: &[u8], len: usize) -> u16 {
    let mut check = InternetChecksum::new(0);
    check.add(&data[..len]);
    check.value()
}

fn compare_tcp_headers_nolen(h1: &TCPHeader, h2: &TCPHeader) -> bool {
    h1.sport == h2.sport
        && h1.dport == h2.dport
        && h1.seqno == h2.seqno
        && h1.ackno == h2.ackno
        && h1.urg == h2.urg
        && h1.ack == h2.ack
        && h1.psh == h2.psh
        && h1.rst == h2.rst
        && h1.syn == h2.syn
        && h1.fin == h2.fin
        && h1.win == h2.win
        && h1.uptr == h2.uptr
}

fn compare_tcp_headers(h1: &TCPHeader, h2: &TCPHeader) -> bool {
    compare_tcp_headers_nolen(h1, h2) && h1.doff == h2.doff
}

#[test]
fn test_tcp_parser() {
    let mut rd = thread_rng();
    for _ in 0..NREPS {
        let mut test_header = vec![0u8; 20];
        rd.fill(&mut test_header[..]);

        // 确保 data offset (doff) 符合要求，同时零初始化校验和字段。
        test_header[12] = 0x50; // 保证 TCP 头部长度为 20 字节
        test_header[16] = 0; // 校验和字段
        test_header[17] = 0;
        let checksum = inet_cksum(&test_header, test_header.len());
        test_header[16] = (checksum >> 8) as u8;
        test_header[17] = (checksum & 0xff) as u8;

        // test parse
        let mut test_1 = TCPHeader::default();
        {
            let buf = Buffer::new_form_vec(test_header.clone());
            let mut p = NetParser::new(buf);
            assert!(test_1.parse(&mut p).is_ok(), "header parse failed");

            let tval = ((test_header[0] as u16) << 8) | (test_header[1] as u16);
            assert_eq!(test_1.sport, tval, "bad parse: wrong source port");

            let tval = ((test_header[2] as u16) << 8) | (test_header[3] as u16);
            assert_eq!(test_1.dport, tval, "bad parse: wrong destination port");

            let tval = ((test_header[4] as u32) << 24)
                | ((test_header[5] as u32) << 16)
                | ((test_header[6] as u32) << 8)
                | (test_header[7] as u32);
            assert_eq!(test_1.seqno.raw_value(), tval, "bad parse: wrong seqno");

            let tval = ((test_header[8] as u32) << 24)
                | ((test_header[9] as u32) << 16)
                | ((test_header[10] as u32) << 8)
                | (test_header[11] as u32);
            assert_eq!(test_1.ackno.raw_value(), tval, "bad parse: wrong ackno");

            let tval = (if test_1.urg { 0x20 } else { 0 })
                | (if test_1.ack { 0x10 } else { 0 })
                | (if test_1.psh { 0x08 } else { 0 })
                | (if test_1.rst { 0x04 } else { 0 })
                | (if test_1.syn { 0x02 } else { 0 })
                | (if test_1.fin { 0x01 } else { 0 });
            assert_eq!(tval, test_header[13] & 0x3f, "bad parse: bad flags");

            let tval = ((test_header[14] as u16) << 8) | (test_header[15] as u16);
            assert_eq!(test_1.win, tval, "bad parse: wrong window value");

            assert_eq!(test_1.cksum, checksum, "bad parse: wrong checksum");

            let tval = ((test_header[18] as u16) << 8) | (test_header[19] as u16);
            assert_eq!(test_1.uptr, tval, "bad parse: wrong urgent pointer");
        }

        test_header[12] = 0x40;
        {
            let new_cksum = inet_cksum(&test_header, test_header.len());
            test_header[16] = (new_cksum >> 8) as u8;
            test_header[17] = (new_cksum & 0xff) as u8;
            let mut p = NetParser::new(Buffer::new_form_vec(test_header.clone()));

            let result = test_1.parse(&mut p);
            assert!(result.is_err(), "header parse should have failed");
            // check result is ParseError::HeaderTooShort?
            assert_eq!(result.unwrap_err(), ParseError::HeaderTooShort);
        }

        test_header[12] = 0x60;
        test_header[16] = (checksum >> 8) as u8;
        test_header[17] = (checksum & 0xff) as u8;
        {
            let mut p = NetParser::new(Buffer::new_form_vec(test_header.clone()));
            let result = test_1.parse(&mut p);
            assert!(
                matches!(result, Err(ParseError::PacketTooShort)),
                "bad parse: got wrong error for segment shorter than 4 * doff"
            );
        }

        // Test segment shorter than 20 bytes
        test_header[12] = 0x50;
        test_header.resize(16, 0);
        {
            let mut p = NetParser::new(Buffer::new_form_vec(test_header));
            let result = test_1.parse(&mut p);
            assert!(
                matches!(result, Err(ParseError::PacketTooShort)),
                "bad parse: got wrong error for segment shorter than 20 bytes"
            );
        }

        // get pcap file: ./ipv4_parser.data
        let filename = "tests/ipv4_parser.data";

        let mut cap = Capture::from_file(&filename).unwrap_or_else(|e| {
            panic!("Error opening file {}: {}", filename, e);
        });

        if cap.get_datalink() != pcap::Linktype(1) {
            panic!("ERROR expected ethernet linktype in capture file");
        }
        let mut ok = true;

        while let Ok(packet) = cap.next_packet() {
            // println!("packet: {:?}", packet);

            let hdr = packet.header;
            // make sure the packet is big enough to contain an Ethernet header
            if hdr.caplen < 14 {
                println!("ERROR frame too short to contain Ethernet header");
                ok = false;
                continue;
            }

            let pkt = packet.data;
            if pkt[12] != 0x08 || pkt[13] != 0x00 {
                continue;
            }

            let hdrlen = ((pkt[14] & 0x0f) << 2) as u8; // TCP header length
            let tlen = ((pkt[16] as u16) << 8) | (pkt[17] as u16); // Total length

            if hdr.caplen - 14 != tlen as u32 {
                continue; // weird! truncated segment
            }

            let tcp_seg_data = &pkt[(14 + (hdrlen as usize))..]; // TCP segment data
            let tcp_seg_len = hdr.caplen as usize - 14 - (hdrlen as usize); // TCP segment length
            let (tcp_seg, result) = {
                let mut tcp_data = tcp_seg_data.to_vec();

                // fix up checksum to remove contribution from IPv4 pseudo-header
                // rust 的 位运算 优先级 低于 + , 与 cpp 不同.
                let mut cksum_fixup = (((pkt[26] as u32) << 8) | (pkt[27] as u32))
                    + (((pkt[28] as u32) << 8) | (pkt[29] as u32)); // src addr
                cksum_fixup += (((pkt[30] as u32) << 8) | (pkt[31] as u32))
                    + (((pkt[32] as u32) << 8) | (pkt[33] as u32)); // dst addr
                cksum_fixup += pkt[23] as u32; // proto
                cksum_fixup += tcp_seg_len as u32; // len
                cksum_fixup += ((tcp_data[16] as u32) << 8) | (tcp_data[17] as u32); // original cksum
                while cksum_fixup > 0xffff {
                    // carry bits
                    cksum_fixup = (cksum_fixup >> 16) + (cksum_fixup & 0xffff);
                }

                tcp_data[16] = (cksum_fixup >> 8) as u8;
                tcp_data[17] = (cksum_fixup & 0xff) as u8;

                let mut tcp_seg_ret: TCPSegment = TCPSegment::default();
                let mut buffer_temp = Buffer::new_form_vec(tcp_data);
                let parse_result = tcp_seg_ret.parse(&mut buffer_temp, 0);
                (tcp_seg_ret, parse_result)
            };

            if result.is_err() {
                println!(
                    "ERROR got unexpected parse failure {:?} for this segment:",
                    result
                );
                hexdump(tcp_seg_data, tcp_seg_len);
                ok = false;
                continue;
            }

            let mut tcp_seg_copy = TCPSegment::default();
            tcp_seg_copy.payload = tcp_seg.payload.clone();
            //  set headers in new segment, and fix up to remove extensions
            {
                let tcp_hdr_orig = &tcp_seg.header;
                let tcp_hdr_copy = &mut tcp_seg_copy.header;
                *tcp_hdr_copy = *tcp_hdr_orig;
                tcp_hdr_copy.doff = 5; // fix up segment to remove IPv4 and TCP header extensions
            } // tcp_hdr_{orig,copy} go out of scope

            assert!(
                compare_tcp_headers_nolen(&tcp_seg.header, &tcp_seg_copy.header),
                "ERROR: after unparsing, TCP headers (other than length) don't match.\n"
            );

            let mut tcp_seg_copy2 = TCPSegment::default();
            let mut buffer_temp =
                Buffer::new_form_vec(tcp_seg_copy.serialize(0).unwrap().concatenate().to_vec());
            let res = tcp_seg_copy2.parse(&mut buffer_temp, 0);

            if res.is_err() {
                println!("ERROR got parse failure {:?} for this segment:", result);
                hexdump(tcp_seg_data, tcp_seg_len);
                ok = false;
                continue;
            }

            if !compare_tcp_headers_nolen(&tcp_seg_copy.header, &tcp_seg_copy2.header) {
                println!("ERROR: after re-parsing, TCP headers don't match.\n");
                ok = false;
                continue;
            }
            if !compare_tcp_headers(&tcp_seg_copy.header, &tcp_seg_copy2.header) {
                println!("ERROR: after re-parsing, TCP headers don't match.\n");
                ok = false;
                continue;
            }

            if tcp_seg_copy.payload.as_slice() != tcp_seg_copy2.payload.as_slice() {
                println!("ERROR: after re-parsing, TCP payloads don't match.\n");
                ok = false;
                continue;
            }
        }

        if !ok {
            process::exit(1);
        }
    }
}
