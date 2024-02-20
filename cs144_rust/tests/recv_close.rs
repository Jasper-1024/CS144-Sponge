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

        let state = ExpectState::new(TCPReceiverStateSummary::Listen);
        test.execute(&state);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();

        test.execute_with_segment(&seg_arrivers, &default_segment);

        let state = ExpectState::new(TCPReceiverStateSummary::SynReceived);
        test.execute(&state);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .fin()
            .seqno((isn + 1).into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();

        test.execute_with_segment(&seg_arrivers, &default_segment);

        let ackno = ExpectAckno::new((isn + 2).into());
        test.execute(&ackno);

        let unassembled_bytes = ExpectUnassembledBytes::new(0);
        test.execute(&unassembled_bytes);

        let bytes = ExpectBytes::new(*b"");
        test.execute(&bytes);

        let total_assembled_bytes = ExpectTotalAssembledBytes::new(0);
        test.execute(&total_assembled_bytes);
    }
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let test = TCPReceiverTestHarness::new(4000);

        let state = ExpectState::new(TCPReceiverStateSummary::Listen);
        test.execute(&state);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();

        test.execute_with_segment(&seg_arrivers, &default_segment);

        let state = ExpectState::new(TCPReceiverStateSummary::SynReceived);
        test.execute(&state);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .fin()
            .seqno((isn + 1).into())
            .data(*b"a")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();

        test.execute_with_segment(&seg_arrivers, &default_segment);

        let state = ExpectState::new(TCPReceiverStateSummary::FinReceived);
        test.execute(&state);

        let ackno = ExpectAckno::new((isn + 3).into());
        test.execute(&ackno);

        let unsn = ExpectUnassembledBytes::new(0);
        test.execute(&unsn);

        let bytes = ExpectBytes::new(*b"a");
        test.execute(&bytes);

        let total_assembled_bytes = ExpectTotalAssembledBytes::new(1);
        test.execute(&total_assembled_bytes);

        let state = ExpectState::new(TCPReceiverStateSummary::FinReceived);
        test.execute(&state);
    }
}
