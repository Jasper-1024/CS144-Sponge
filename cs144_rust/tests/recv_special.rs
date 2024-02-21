mod receiver_harness;
use cs144_rust::tcp_helpers::tcp_state::TCPReceiverStateSummary;
use rand::Rng;
use receiver_harness::*;

#[test]
fn test_recv_special() {
    let mut rng = rand::thread_rng();

    /* segment before SYN */
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let test = TCPReceiverTestHarness::new(4000);
        execute_test!(test, ExpectState, TCPReceiverStateSummary::Listen);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 1).into())
            .data(*b"hello")
            .result(Some(Result::NotSyn))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectState, TCPReceiverStateSummary::Listen);
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectBytes, *b"");
        execute_test!(test, ExpectTotalAssembledBytes, 0);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectState, TCPReceiverStateSummary::SynReceived);
        execute_test!(test, ExpectAckno, (isn + 1).into());
    }

    /* segment with SYN + data */
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let test = TCPReceiverTestHarness::new(4000);
        execute_test!(test, ExpectState, TCPReceiverStateSummary::Listen);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .data(*b"Hello, CS144!")
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectState, TCPReceiverStateSummary::SynReceived);
        execute_test!(test, ExpectAckno, (isn + 14).into());
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectBytes, *b"Hello, CS144!");
    }

    /* empty segment */
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
        execute_test!(test, ExpectAckno, (isn + 1).into());
        execute_test!(test, ExpectUnassembledBytes, 0);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno((isn + 1).into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 0);
        execute_test!(test, ExpectInputNotEnded, true);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno((isn + 5).into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 0);
        execute_test!(test, ExpectInputNotEnded, true);
    }

    /* segment with null byte */
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
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectTotalAssembledBytes, 0);

        let text = b"Here's a null byte:\0and it's gone.";
        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .seqno((isn + 1).into())
            .data(*text)
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectBytes, *text);
        execute_test!(test, ExpectAckno, (isn + 35).into());
        execute_test!(test, ExpectInputNotEnded, true);
    }

    /* segment with data + FIN */
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
            .data(*b"Goodbye, CS144!")
            .seqno((isn + 1).into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectState, TCPReceiverStateSummary::FinReceived);
        execute_test!(test, ExpectBytes, *b"Goodbye, CS144!");
        execute_test!(test, ExpectAckno, (isn + 17).into());
        execute_test!(test, ExpectEof, true);
    }

    /* segment with FIN (but can't be assembled yet) */
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
            .data(*b"oodbye, CS144!")
            .seqno((isn + 2).into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectState, TCPReceiverStateSummary::SynReceived);
        execute_test!(test, ExpectBytes, *b"");
        execute_test!(test, ExpectAckno, (isn + 1).into());
        execute_test!(test, ExpectInputNotEnded, true);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .data(*b"G")
            .seqno((isn + 1).into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectState, TCPReceiverStateSummary::FinReceived);
        execute_test!(test, ExpectBytes, *b"Goodbye, CS144!");
        execute_test!(test, ExpectAckno, (isn + 17).into());
        execute_test!(test, ExpectEof, true);
    }

    /* segment with SYN + data + FIN */
    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);
        let test = TCPReceiverTestHarness::new(4000);
        execute_test!(test, ExpectState, TCPReceiverStateSummary::Listen);

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .data(*b"Hello and goodbye, CS144!")
            .fin()
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();
        test.execute_with_segment(&seg_arrivers, &default_segment);

        execute_test!(test, ExpectState, TCPReceiverStateSummary::FinReceived);
        execute_test!(test, ExpectAckno, (isn + 27).into());
        execute_test!(test, ExpectUnassembledBytes, 0);
        execute_test!(test, ExpectBytes, *b"Hello and goodbye, CS144!");
        execute_test!(test, ExpectEof, true);
    }
}
