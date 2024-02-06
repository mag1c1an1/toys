trait LendingIterator {
    type Item<'a>
    where
        Self: 'a;
}

pub struct WindowsMut<'x, T> {
    slice: &'x mut [T],
}

impl<'x, T> LendingIterator for WindowsMut<'x, T> {
    type Item<'a> = &'a mut [T] where Self:'a;
}

fn print_item<I>(iter: I)
where
    I: LendingIterator,
    for<'a> I::Item<'a>: std::fmt::Debug,
{
}

fn main() {
    println!(">>>")
}
