use cs144_rust::tcp_helpers::tcp_state::TCPReceiverStateSummary;
use rand::Rng;

mod receiver_harness;
use receiver_harness::*;

#[test]
fn test_recv_close() {
    let mut rng = rand::thread_rng();

    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let test = TCPReceiverTestHarness::new(4000);

        execute_test!(test, ExpectState, TCPReceiverStateSummary::Listen);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectState, TCPReceiverStateSummary::SynReceived);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .fin()
            .seqno((isn + 1).into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectAckno, (isn + 2).into());
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectBytes, b"".to_vec());
        execute_test!(test, ExpectTotalAssembledBytes, 0);
        execute_test!(test, ExpectState, TCPReceiverStateSummary::FinReceived);
    }
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let test = TCPReceiverTestHarness::new(4000);

        execute_test!(test, ExpectState, TCPReceiverStateSummary::Listen);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectState, TCPReceiverStateSummary::SynReceived);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .fin()
            .seqno((isn + 1).into())
            .data(*b"a")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectState, TCPReceiverStateSummary::FinReceived);
        execute_test!(test, ExpectAckno, (isn + 3).into());
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectBytes, b"a".to_vec());
        execute_test!(test, ExpectTotalAssembledBytes, 1);
        execute_test!(test, ExpectState, TCPReceiverStateSummary::FinReceived);
    }
}
