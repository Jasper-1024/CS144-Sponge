mod receiver_harness;
use cs144_rust::{tcp_helpers::tcp_state::TCPReceiverStateSummary, util::buffer};
use rand::Rng;
use receiver_harness::*;

#[test]
fn test_recv_special() {
    let mut rng = rand::thread_rng();

    {
        let test = TCPReceiverTestHarness::new(4000);
        let isn = 0;

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 1).into())
            .data(*b"abcd")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 5).into());
        execute_test!(test, ExpectBytes, b"abcd".to_vec());
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 4);
    }

    {
        let isn = 384678;
        let test = TCPReceiverTestHarness::new(4000);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 1).into())
            .data(*b"abcd")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 5).into());
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 4);
        execute_test!(test, ExpectBytes, b"abcd".to_vec());

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 5).into())
            .data(*b"efgh")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 9).into());
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 8);
        execute_test!(test, ExpectBytes, b"efgh".to_vec());
    }

    {
        let isn = 5;
        let test = TCPReceiverTestHarness::new(4000);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 1).into())
            .data(*b"abcd")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 5).into());
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 4);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 5).into())
            .data(*b"efgh")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 9).into());
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 8);
        execute_test!(test, ExpectBytes, b"abcdefgh".to_vec());
    }

    // Many (arrive/read)s
    {
        let test = TCPReceiverTestHarness::new(4000);
        let max_block_size = 10;
        let n_rounds = 10000;
        let isn = 893472;
        let mut bytes_sent: usize = 0;

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        let mut buffer = Vec::new();

        for i in 0..n_rounds {
            let mut data = Vec::new();
            let block_size: u32 = rng.gen_range(1..=max_block_size);
            for j in 0..block_size {
                let c = b'a' + ((i + j) % 26) as u8;
                data.push(c);
            }

            let ex_ack = ExpectAckno::new(Some(isn + bytes_sent as u32 + 1));
            let ex_ab = ExpectTotalAssembledBytes::new(bytes_sent);

            let mut seg_arrivers = SegmentArrivesBuilder::new()
                .seqno((isn + bytes_sent as u32 + 1).into())
                .data_vec(data.clone())
                .result(Some(Result::Ok))
                .build();
            let default_segment = seg_arrivers.build_segment();

            bytes_sent += block_size as usize;

            let exbytes = ExpectBytes::new(data.clone());

            buffer.push((ex_ack, ex_ab, seg_arrivers, default_segment, exbytes));
        }

        for value in &buffer {
            let (ex_ack, ex_ab, seg_arrivers, default_segment, exbytes) = value;
            test.execute(ex_ack);
            test.execute(ex_ab);
            test.execute_with_segment(seg_arrivers, &default_segment);
            test.execute(exbytes);
        }
    }

    {
        let max_block_size = 10;
        let n_rounds = 100;
        let mut test = TCPReceiverTestHarness::new(((max_block_size * n_rounds) as u16).into());
        let isn = 238;
        let mut bytes_sent = 0;
        let mut all_data = Vec::new();

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        let mut buffer = Vec::new();

        for i in 0..n_rounds {
            let mut data = Vec::new();
            let block_size: u32 = rng.gen_range(1..=max_block_size);
            for j in 0..block_size {
                let c = b'a' + ((i + j) % 26) as u8;
                data.push(c);
                all_data.push(c);
            }

            let ex_ack = ExpectAckno::new(Some(isn + bytes_sent as u32 + 1));
            let ex_ab = ExpectTotalAssembledBytes::new(bytes_sent);

            let mut seg_arrivers = SegmentArrivesBuilder::new()
                .seqno((isn + bytes_sent as u32 + 1).into())
                .data_vec(data.clone())
                .result(Some(Result::Ok))
                .build();
            let default_segment = seg_arrivers.build_segment();

            bytes_sent += block_size as usize;

            buffer.push((ex_ack, ex_ab, seg_arrivers, default_segment));
        }

        for value in &buffer {
            let (ex_ack, ex_ab, seg_arrivers, default_segment) = value;
            test.execute(ex_ack);
            test.execute(ex_ab);
            test.execute_with_segment(seg_arrivers, &default_segment);
        }

        execute_test!(test, ExpectBytes, all_data);

        // for i in 0..n_rounds {
        //     let mut data = Vec::new();
        //     let block_size: u32 = rng.gen_range(1..=max_block_size);
        //     for j in 0..block_size {
        //         let c = b'a' + ((i + j) % 26) as u8;
        //         data.push(c);
        //         all_data.push(c);
        //     }
        //     execute_test!(test, ExpectAckno, (isn + bytes_sent as u32 + 1).into());
        //     execute_test!(test, ExpectTotalAssembledBytes, bytes_sent);
        //     let mut seg_arrivers = SegmentArrivesBuilder::new()
        //         .seqno((isn + bytes_sent as u32 + 1).into())
        //         .data(data.clone())
        //         .result(Some(Result::Ok))
        //         .build();
        //     let default_segment = seg_arrivers.build_segment();
        //     test.execute_with_segment(&seg_arrivers, &default_segment);

        //     bytes_sent += block_size as usize;
        // }
    }
}
