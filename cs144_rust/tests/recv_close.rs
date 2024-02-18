use std::{borrow::BorrowMut, cell::RefCell, rc::Rc};

use cs144_rust::{
    tcp_helpers::{tcp_segment::TCPSegment, tcp_state::TCPReceiverStateSummary},
    tcp_receiver::TCPReceiver,
};
use rand::Rng;

mod receiver_harness;
use receiver_harness::*;

#[test]
fn test_recv_close() {
    let mut rng = rand::thread_rng();

    {
        let isn: u32 = rng.gen_range(0..=std::u32::MAX);

        let receiver = Rc::new(RefCell::new(TCPReceiver::new(4000)));

        ExpectState::new(TCPReceiverStateSummary::Listen).execute(receiver.clone());

        // let mut test = TCPReceiverTestHarness::new(4000);

        // {
        //     let data = Rc::new(RefCell::new(5));
        //     let data_ref = Rc::clone(&data);

        //     *data_ref.borrow_mut() = 1;
        // }

        // test.execute(&ExpectState::new(TCPReceiverStateSummary::Listen));

        // Rc::get_mut(&mut test)
        //     .unwrap()
        //     .execute(&ExpectState::new(TCPReceiverStateSummary::Listen));

        // let default_segment = TCPSegment::default();
        // let mut seg_arrivers = SegmentArrives::default(&default_segment);
        // seg_arrivers.syn = true;
        // seg_arrivers.seqno = isn.into();
        // seg_arrivers.result = Some(Result::Ok);
        // let default_segment = seg_arrivers.build_segment();
        // seg_arrivers.tcp_segment = &default_segment;

        let mut seg_arrivers = SegmentArrivesBuilder::new()
            .syn()
            .seqno(isn.into())
            .result(Some(Result::Ok))
            .build();
        let default_segment = seg_arrivers.build_segment();

        seg_arrivers.execute_with_segment(receiver.clone(), &default_segment);

        // test.execute_with_segment(&seg_arrivers, &default_segment);
    }
}
