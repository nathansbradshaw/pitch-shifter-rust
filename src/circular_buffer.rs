const BUFFER_SIZE: usize = 30000;

pub struct CircularBuffer<T> {
    buffer: [T; BUFFER_SIZE],
    read_index: usize,
    write_index: usize,
    hop_pointer: usize,
    hop_size: usize,
    default_value: T,
}

impl<T> CircularBuffer<T>
where
    T: Copy + core::ops::AddAssign +std::fmt::Debug,
{
    pub fn new(
        default_value: T,
        hop_size: Option<usize>,
    ) -> CircularBuffer<T> {
        let hop_size = match hop_size {
            Some(value) => value,
            None => 0,
        };


        CircularBuffer {
            buffer: [default_value; BUFFER_SIZE],
            read_index: 0,
            write_index: hop_size,
            hop_size: hop_size,
            hop_pointer: hop_size,
            default_value,
        }
    }

    fn increment_index(&mut self, mut index: usize) -> usize {
        index += 1;

        if index >= self.buffer.len() {
            index = 0;
        }

        index
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
      println!("index before hop {}, after {}", self.hop_pointer, hop_index);
      self.hop_pointer = hop_index;
      self.write_index = hop_index;

    }

    pub fn push_read_back(&mut self, window_size: usize) {
        let push_back = ((self.read_index as isize - window_size as isize + BUFFER_SIZE as isize) % BUFFER_SIZE as isize) as usize;
        println!("read index before {}, {}, after {}",window_size ,self.read_index, push_back);
        self.read_index = push_back;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_push_back() {
        let mut buffer: CircularBuffer<f32> = CircularBuffer::new(0.0, Some(BUFFER_SIZE - 4));
        assert_eq!(buffer.read_index, 0);
        buffer.push_read_back(1);
        assert_eq!(buffer.read_index, BUFFER_SIZE -1);
        buffer.read();
        buffer.read();
        assert_eq!(buffer.read_index, 1);
        buffer.push_read_back(12);
        assert_eq!(buffer.read_index, BUFFER_SIZE -11);
    }

    #[test]
    fn test_hop_size() {
        let mut buffer: CircularBuffer<f32> = CircularBuffer::new(0.0, Some(BUFFER_SIZE - 4));
        assert_eq!(buffer.hop_pointer, BUFFER_SIZE - 4);
        assert_eq!(buffer.write_index, BUFFER_SIZE - 4);
        buffer.next_hop();
        assert_eq!(buffer.write_index, BUFFER_SIZE - 8);
        assert_eq!(buffer.hop_pointer, BUFFER_SIZE - 8);

    }

    #[test]
    fn test_initialization() {
        let buffer: CircularBuffer<f32> = CircularBuffer::new(0.0, None);
        for &item in buffer.buffer.iter() {
            assert_eq!(item, 0.0);
        }
    }

    #[test]
    fn test_write_and_read() {
        let mut buffer = CircularBuffer::new(0, None);
        buffer.write(1);
        buffer.write(2);
        assert_eq!(buffer.read(), 1);
        assert_eq!(buffer.read(), 2);
    }

    #[test]
    fn test_wraparound() {
        let mut buffer = CircularBuffer::new(0, None);
        for i in 0..BUFFER_SIZE {
            buffer.write(i as i32);
        }
        buffer.write(1024);
        assert_eq!(buffer.read(), 1024); // Reading the overwritten first element
        assert_eq!(buffer.write_index, 1);
        assert_eq!(buffer.read_index, 1);
    }

    #[test]
    fn test_read_and_reset() {
        let mut buffer = CircularBuffer::new(0, None);
        buffer.write(1);
        assert_eq!(buffer.read_and_reset(), 1);
        assert_eq!(buffer.buffer[0], 0); // Ensure the value is reset to default
    }

    #[test]
    fn test_add_value() {
        let mut buffer = CircularBuffer::new(0, None);
        buffer.add_value(1);
        assert_eq!(buffer.buffer[0], 1);
        buffer.add_value(2);
        assert_eq!(buffer.buffer[1], 2);
        buffer.add_value(3);
        assert_eq!(buffer.buffer[2], 3);
    }
}
