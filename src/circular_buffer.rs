
pub struct CircularBuffer<T> {
    buffer: [T; BUFFER_SIZE],
    head: usize,
    tail: usize,
    full: bool,
    count: usize,
    hop_size: usize,
}

const BUFFER_SIZE: usize = 10; 

impl<T: Copy + Default> CircularBuffer<T> {
    pub fn new(hop_size: usize) -> Self {
        CircularBuffer {
            buffer: [T::default(); BUFFER_SIZE],
            head: 0,
            tail: 0,
            full: false,
            count: 0,
            hop_size,
        }
    }

    pub fn push(&mut self, item: T) {
        self.buffer[self.head] = item;
        self.head = (self.head + 1) % BUFFER_SIZE;

        if self.full {
            self.tail = (self.tail + 1) % BUFFER_SIZE;
        }

        if self.head == self.tail {
            self.full = true;
        }

        self.count += 1;

        if self.count >= self.hop_size {
            self.process_window();
            self.count = 0;
        }
    }

    fn process_window(&self) {
        // Process the buffer window
        // For example, copy the data to a new buffer and process it
        let mut window = vec![T::default(); BUFFER_SIZE];
        for i in 0..BUFFER_SIZE {
            window[i] = self.buffer[(self.tail + i) % BUFFER_SIZE];
        }
        // Here you can pass the window to your FFT function or any other processing function
        // Example:
        // fft_process(&window);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            let item = Some(self.buffer[self.tail]);
            self.tail = (self.tail + 1) % BUFFER_SIZE;
            self.full = false;
            item
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head == self.tail && !self.full
    }

    pub fn is_full(&self) -> bool {
        self.full
    }

    pub fn len(&self) -> usize {
        if self.full {
            BUFFER_SIZE
        } else if self.head >= self.tail {
            self.head - self.tail
        } else {
            BUFFER_SIZE - self.tail + self.head
        }
    }

    pub fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.full = false;
        self.buffer = [T::default(); BUFFER_SIZE];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_pop() {
        let mut buffer = CircularBuffer::new(5);

        buffer.push(1);
        buffer.push(2);
        buffer.push(3);

        assert_eq!(buffer.pop(), Some(1));
        assert_eq!(buffer.pop(), Some(2));
        assert_eq!(buffer.pop(), Some(3));
        assert_eq!(buffer.pop(), None);
    }

    #[test]
    fn test_buffer_wrap_around() {
        let mut buffer = CircularBuffer::new(5);

        for i in 0..BUFFER_SIZE {
            buffer.push(i);
        }

        for i in 0..BUFFER_SIZE {
            assert_eq!(buffer.pop(), Some(i));
        }
        assert_eq!(buffer.pop(), None);
    }

    #[test]
    fn test_is_empty_and_is_full() {
        let mut buffer = CircularBuffer::new(5);

        assert!(buffer.is_empty());
        assert!(!buffer.is_full());

        for i in 0..BUFFER_SIZE {
            buffer.push(i);
        }

        assert!(!buffer.is_empty());
        assert!(buffer.is_full());

        for _ in 0..BUFFER_SIZE {
            buffer.pop();
        }

        assert!(buffer.is_empty());
        assert!(!buffer.is_full());
    }

    #[test]
    fn test_len() {
        let mut buffer = CircularBuffer::new(5);

        assert_eq!(buffer.len(), 0);

        for i in 0..5 {
            buffer.push(i);
            assert_eq!(buffer.len(), i + 1);
        }

        for i in 0..5 {
            buffer.pop();
            assert_eq!(buffer.len(), 4 - i);
        }
    }

    #[test]
    fn test_clear() {
        let mut buffer = CircularBuffer::new(5);

        for i in 0..5 {
            buffer.push(i);
        }

        buffer.clear();

        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.pop(), None);
    }

    #[test]
    fn test_process_window() {
        // Since process_window is private, we'll need to simulate it by pushing enough items
        // Create a custom buffer to count how many times process_window is called
        struct TestBuffer {
            buffer: CircularBuffer<i32>,
            process_count: usize,
        }

        impl TestBuffer {
            fn new(hop_size: usize) -> Self {
                TestBuffer {
                    buffer: CircularBuffer::new(hop_size),
                    process_count: 0,
                }
            }

            fn push(&mut self, item: i32) {
                self.buffer.push(item);
                if self.buffer.count == 0 {
                    self.process_count += 1;
                }
            }

            fn process_count(&self) -> usize {
                self.process_count
            }
        }

        let mut test_buffer = TestBuffer::new(5);

        for i in 0..10 {
            test_buffer.push(i);
        }

        // process_window should be called twice because hop_size is 5 and we pushed 10 items
        assert_eq!(test_buffer.process_count(), 2);
    }
}
