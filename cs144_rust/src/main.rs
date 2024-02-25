use std::{cell::RefCell, rc::Rc};

fn main() {
    // 创建一个包含usize值的RefCell，并用Rc包装，以便可以共享
    let shared_value = Rc::new(RefCell::new(5));

    // 创建第一个引用 clone1
    let clone1 = Rc::clone(&shared_value);

    // 创建第二个引用 clone2
    let clone2 = Rc::clone(&shared_value);

    // 通过clone1来修改内部的usize值
    *clone1.borrow_mut() += 10;
    
    // 打印当前值，通过clone2来查看修改
    println!("Current value: {}", clone2.borrow());

    // 再次通过clone1来修改内部的usize值
    *clone1.borrow_mut() *= 2;

    // 最后再次打印当前值，通过原始shared_value来查看修改
    println!("Current value after second modification: {}", shared_value.borrow());
}
