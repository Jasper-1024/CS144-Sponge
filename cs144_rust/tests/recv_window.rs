mod receiver_harness;
use cs144_rust::tcp_helpers::tcp_state::TCPReceiverStateSummary;
use rand::Rng;
use receiver_harness::*;

#[test]
fn test_recv_special() {
    {
        // Window size decreases appropriately
        let cap: usize = 4000;
        let isn: u32 = 23452;
        let mut test = TCPReceiverTestHarness::new(cap);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 1).into());
        execute_test!(test, ExpectWindow, cap);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 1).into())
            .data(*b"abcd")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 5).into());
        execute_test!(test, ExpectWindow, (cap - 4));

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 9).into())
            .data(*b"ijkl")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 5).into());
        execute_test!(test, ExpectWindow, (cap - 4));

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 5).into())
            .data(*b"efgh")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 13).into());
        execute_test!(test, ExpectWindow, (cap - 12));
    }

    {
        // Window size expands upon read
        let cap: usize = 4000;
        let isn: u32 = 23452;
        let mut test = TCPReceiverTestHarness::new(cap);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 1).into());
        execute_test!(test, ExpectWindow, cap);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 1).into())
            .data(*b"abcd")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 5).into());
        execute_test!(test, ExpectWindow, cap - 4);

        execute_test!(test, ExpectBytes, b"abcd".to_vec());
        execute_test!(test, ExpectAckno, (isn + 5).into());
        execute_test!(test, ExpectWindow, cap);
    }

    {
        // almost-high-seqno segment is accepted, but only some bytes are kept
        let cap: usize = 2;
        let isn: u32 = 23452;
        let mut test = TCPReceiverTestHarness::new(cap);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 2).into())
            .data(*b"bc")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectTotalAssembledBytes, 0);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 1).into())
            .data(*b"a")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 3).into());
        execute_test!(test, ExpectWindow, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 2);
        execute_test!(test, ExpectBytes, b"ab".to_vec());
        execute_test!(test, ExpectWindow, cap);
    }

    {
        // Segment overflowing the window on left side is acceptable.
        let cap: usize = 4;
        let isn: u32 = 23452;
        let mut test = TCPReceiverTestHarness::new(cap);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 1).into())
            .data(*b"ab")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 3).into())
            .data(*b"cdef")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);
    }

    {
        // Segment matching the window is acceptable.
        let cap: usize = 4;
        let isn: u32 = 23452;
        let mut test = TCPReceiverTestHarness::new(cap);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 1).into())
            .data(*b"ab")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 3).into())
            .data(*b"cd")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);
    }

    {
        // A byte with invalid stream index should be ignored
        let cap: usize = 4;
        let isn: u32 = 23452;
        let test = TCPReceiverTestHarness::new(cap);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno(isn.into())
            .data(*b"a")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectTotalAssembledBytes, 0);
    }
}
