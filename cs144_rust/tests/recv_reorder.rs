mod receiver_harness;
use rand::Rng;
use receiver_harness::*;

#[test]
fn test_recv_reorder() {
    let mut rng = rand::thread_rng();

    // An in-window, but later segment
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let test = TCPReceiverTestHarness::new(2358);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 10).into())
            .data(*b"abcd")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 1).into());
        execute_test!(test, ExpectBytes, b"".to_vec());
        execute_test!(test, ExpectUnassembledBytes, 4);
        execute_test!(test, ExpectTotalAssembledBytes, 0);
    }

    // An in-window, but later segment, then the hole is filled
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let test = TCPReceiverTestHarness::new(2358);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 1).into());

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 5).into())
            .data(*b"efgh")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 1).into());
        execute_test!(test, ExpectBytes, b"".to_vec());
        execute_test!(test, ExpectUnassembledBytes, 4);
        execute_test!(test, ExpectTotalAssembledBytes, 0);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 1).into())
            .data(*b"abcd")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 9).into());
        execute_test!(test, ExpectBytes, b"abcdefgh".to_vec());
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 8);
    }

    // An in-window, but later segment, then the hole is filled, bit by bit
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let test = TCPReceiverTestHarness::new(2358);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 1).into());

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 5).into())
            .data(*b"efgh")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 1).into());
        execute_test!(test, ExpectBytes, b"".to_vec());
        execute_test!(test, ExpectUnassembledBytes, 4);
        execute_test!(test, ExpectTotalAssembledBytes, 0);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 1).into())
            .data(*b"ab")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 3).into());
        execute_test!(test, ExpectBytes, b"ab".to_vec());
        execute_test!(test, ExpectUnassembledBytes, 4);
        execute_test!(test, ExpectTotalAssembledBytes, 2);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 3).into())
            .data(*b"cd")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 9).into());
        execute_test!(test, ExpectBytes, b"cdefgh".to_vec());
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 8);
    }

    // Many gaps, then filled bit by bit.
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let test = TCPReceiverTestHarness::new(2358);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 1).into());

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 5).into())
            .data(*b"e")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 1).into());
        execute_test!(test, ExpectBytes, b"".to_vec());
        execute_test!(test, ExpectUnassembledBytes, 1);
        execute_test!(test, ExpectTotalAssembledBytes, 0);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 7).into())
            .data(*b"g")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 1).into());
        execute_test!(test, ExpectBytes, b"".to_vec());
        execute_test!(test, ExpectUnassembledBytes, 2);
        execute_test!(test, ExpectTotalAssembledBytes, 0);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 3).into())
            .data(*b"c")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 1).into());
        execute_test!(test, ExpectBytes, b"".to_vec());
        execute_test!(test, ExpectUnassembledBytes, 3);
        execute_test!(test, ExpectTotalAssembledBytes, 0);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 1).into())
            .data(*b"ab")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 4).into());
        execute_test!(test, ExpectBytes, b"abc".to_vec());
        execute_test!(test, ExpectUnassembledBytes, 2);
        execute_test!(test, ExpectTotalAssembledBytes, 3);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 6).into())
            .data(*b"f")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectUnassembledBytes, 3);
        execute_test!(test, ExpectTotalAssembledBytes, 3);
        execute_test!(test, ExpectBytes, b"".to_vec());

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 4).into())
            .data(*b"d")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 7);
        execute_test!(test, ExpectBytes, b"defg".to_vec());
    }

    // Many gaps, then subsumed
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let test = TCPReceiverTestHarness::new(2358);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 1).into());

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 5).into())
            .data(*b"e")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 1).into());
        execute_test!(test, ExpectBytes, b"".to_vec());
        execute_test!(test, ExpectUnassembledBytes, 1);
        execute_test!(test, ExpectTotalAssembledBytes, 0);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 7).into())
            .data(*b"g")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 1).into());
        execute_test!(test, ExpectBytes, b"".to_vec());
        execute_test!(test, ExpectUnassembledBytes, 2);
        execute_test!(test, ExpectTotalAssembledBytes, 0);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 3).into())
            .data(*b"c")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 1).into());
        execute_test!(test, ExpectBytes, b"".to_vec());
        execute_test!(test, ExpectUnassembledBytes, 3);
        execute_test!(test, ExpectTotalAssembledBytes, 0);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 1).into())
            .data(*b"abcdefgh")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 9).into());
        execute_test!(test, ExpectBytes, b"abcdefgh".to_vec());
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 8);
    }
}
