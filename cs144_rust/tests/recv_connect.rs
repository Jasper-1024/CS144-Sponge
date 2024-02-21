mod receiver_harness;
use receiver_harness::*;

#[test]
fn test_recv_connect() {
    {
        let test = TCPReceiverTestHarness::new(4000);

        execute_test!(test, ExpectWindow, 4000);
        execute_test!(test, ExpectAckno, None);
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 0);

        let mut seg_arrives = SegmentArrivesBuilder::new()
            .syn()
            .seqno(0)
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrives.build_segment();
        test.execute_with_segment(&seg_arrives, &default_segment);

        execute_test!(test, ExpectAckno, Some(1));
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 0);
    }

    {
        let test = TCPReceiverTestHarness::new(5435);

        execute_test!(test, ExpectAckno, None);
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 0);

        let mut seg_arrives = SegmentArrivesBuilder::new()
            .syn()
            .seqno(89347598)
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrives.build_segment();
        test.execute_with_segment(&seg_arrives, &default_segment);

        execute_test!(test, ExpectAckno, Some(89347599));
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 0);
    }

    {
        let test = TCPReceiverTestHarness::new(5435);

        execute_test!(test, ExpectAckno, None);
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 0);

        let mut seg_arrives = SegmentArrivesBuilder::new()
            .seqno(893475)
            .result(Some(Result::NotSyn))
            .build();
        let default_segment = seg_arrives.build_segment();
        test.execute_with_segment(&seg_arrives, &default_segment);

        execute_test!(test, ExpectAckno, None);
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 0);
    }

    {
        let test = TCPReceiverTestHarness::new(5435);

        execute_test!(test, ExpectAckno, None);
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 0);

        let mut seg_arrives = SegmentArrivesBuilder::new()
            .ack(0)
            .fin()
            .seqno(893475)
            .result(Some(Result::NotSyn))
            .build();
        let default_segment = seg_arrives.build_segment();
        test.execute_with_segment(&seg_arrives, &default_segment);

        execute_test!(test, ExpectAckno, None);
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 0);
    }
    {
        let test = TCPReceiverTestHarness::new(5435);

        execute_test!(test, ExpectAckno, None);
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 0);

        let mut seg_arrives = SegmentArrivesBuilder::new()
            .ack(0)
            .fin()
            .seqno(893475)
            .result(Some(Result::NotSyn))
            .build();
        let default_segment = seg_arrives.build_segment();
        test.execute_with_segment(&seg_arrives, &default_segment);

        execute_test!(test, ExpectAckno, None);
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 0);

        let mut seg_arrives = SegmentArrivesBuilder::new()
            .syn()
            .seqno(89347598)
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrives.build_segment();
        test.execute_with_segment(&seg_arrives, &default_segment);

        execute_test!(test, ExpectAckno, Some(89347599));
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 0);
    }

    {
        let cap = u16::MAX as usize + 5;
        let test = TCPReceiverTestHarness::new(cap);
        execute_test!(test, ExpectWindow, cap);
    }
}
