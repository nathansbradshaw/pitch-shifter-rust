pub struct CircularBuffer<T, const N: usize> {
    buffer: [T; N],
    read_index: usize,
    write_index: usize,
    hop_pointer: usize,
    hop_size: usize,
    default_value: T,
}

impl<T, const N: usize> CircularBuffer<T, N>
where
    T: Copy + core::ops::AddAssign + core::fmt::Debug,
{
    pub fn new(default_value: T, hop_size: Option<usize>) -> CircularBuffer<T, N> {
        let hop_size = hop_size.unwrap_or(0);

        CircularBuffer {
            buffer: [default_value; N],
            read_index: 0,
            write_index: 0,
            hop_size,
            hop_pointer: 0,
            default_value,
        }
    }

    fn increment_index(&mut self, index: usize) -> usize {
        (index + 1) % N
    }

    pub fn read(&mut self) -> T {
        let current_index = self.read_index;
        self.read_index = self.increment_index(self.read_index);

        self.buffer[current_index]
    }

    pub fn write(&mut self, value: T) -> () {
        self.buffer[self.write_index] = value;

        //if we are at the max buffer size, circle back to 0
        self.write_index = self.increment_index(self.write_index);
    }

    pub fn read_and_reset(&mut self) -> T {
        // Check that read isn't past hop pointer
        let value = self.buffer[self.read_index];
        self.buffer[self.read_index] = self.default_value;

        self.read_index = self.increment_index(self.read_index);

        value
    }

    pub fn add_value(&mut self, value: T) {
        self.buffer[self.write_index] += value;
        self.write_index = self.increment_index(self.write_index);
    }

    pub fn next_hop(&mut self) {
      let hop_index = (self.hop_pointer + self.hop_size) % self.buffer.len();
      self.hop_pointer = hop_index;
      self.write_index = hop_index;

    }

    // push the read pointer back a window (this assumes we have already moved the hop index one hop)
    pub fn push_read_pointer_back(&mut self, window_size: usize) {
        let push_back = ((self.read_index as isize - window_size as isize + self.buffer.len() as isize) % self.buffer.len() as isize) as usize;
        self.read_index = push_back;
    }
}

// TODO add test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let buffer: CircularBuffer<i32, 5> = CircularBuffer::new(0, Some(2));
        assert_eq!(buffer.buffer, [0, 0, 0, 0, 0]);
        assert_eq!(buffer.read_index, 0);
        assert_eq!(buffer.write_index, 0);
        assert_eq!(buffer.hop_size, 2);
        assert_eq!(buffer.hop_pointer, 0);
    }

    #[test]
    fn test_write_and_read() {
        let mut buffer: CircularBuffer<i32, 3> = CircularBuffer::new(0, None);
        buffer.write(1);
        buffer.write(2);
        buffer.write(3);

        assert_eq!(buffer.read(), 1);
        assert_eq!(buffer.read(), 2);
        assert_eq!(buffer.read(), 3);
    }

    #[test]
    fn test_write_overflow() {
        let mut buffer: CircularBuffer<i32, 3> = CircularBuffer::new(0, None);
        buffer.write(1);
        buffer.write(2);
        buffer.write(3);
        buffer.write(4);

        assert_eq!(buffer.read(), 2); // 1 is overwritten
        assert_eq!(buffer.read(), 3);
        assert_eq!(buffer.read(), 4);
    }

    #[test]
    fn test_read_and_reset() {
        let mut buffer: CircularBuffer<i32, 3> = CircularBuffer::new(0, None);
        buffer.write(1);
        buffer.write(2);

        assert_eq!(buffer.read_and_reset(), 1);
        assert_eq!(buffer.read_and_reset(), 2);
        assert_eq!(buffer.read_and_reset(), 0); // reset value
    }

    #[test]
    fn test_add_value() {
        let mut buffer: CircularBuffer<i32, 3> = CircularBuffer::new(0, None);
        buffer.write(1);
        buffer.write(2);
        buffer.add_value(3);

        assert_eq!(buffer.read(), 1);
        assert_eq!(buffer.read(), 2);
        assert_eq!(buffer.read(), 3); // 0 + 3
    }

    #[test]
    fn test_next_hop() {
        let mut buffer: CircularBuffer<i32, 5> = CircularBuffer::new(0, Some(2));
        buffer.write(1);
        buffer.write(2);
        buffer.next_hop();

        assert_eq!(buffer.read_index, 0);
        assert_eq!(buffer.write_index, 2);
        assert_eq!(buffer.hop_pointer, 2);
    }

    #[test]
    fn test_push_read_back() {
        let mut buffer: CircularBuffer<i32, 5> = CircularBuffer::new(0, None);

        //write
        for i in 1..=5 {
            buffer.write(i);
            println!("{}", i);
        }

        buffer.push_read_pointer_back(2);
        assert_eq!(buffer.read_index+1, 4);
        
    }


    //TODO: add in logic for hitting a hop size first
    #[test]
    fn test_read_index_integrety() {
        let mut buffer: CircularBuffer<i32, 5> = CircularBuffer::new(0, None);

        for i in 1..=5 {
            buffer.write(i);
            println!("{}", i);
        }

        buffer.push_read_pointer_back(2);
        for i in 1..=5 {
            let mut res = buffer.read();
            println!("{}", res);
        }

        for i in 1..=5 {
            buffer.write(i);
            println!("{}", i);
        }

        buffer.push_read_pointer_back(2);
        for i in 1..=5 {
            let mut res = buffer.read();
            println!("{}", res);
        }

        for i in 1..=5 {
            buffer.write(i);
            println!("{}", i);
        }

        buffer.push_read_pointer_back(2);
        for i in 1..=5 {
            let mut res = buffer.read();
            println!("{}", res);
        }

        assert_eq!(buffer.read_index+1, 4);
    }
}
