# Fixed-size priority buffer

A fixed-size priority buffer is a fixed-size FIFO queue which maintains it's size constraint by evicting lower priority elements.

## Applications

In a senor network, it is quite common that devices with constrained resources are collecting large quantities of data to upload to a central server for storage and processing. Commonly these could have unreliable connections and would need to use a buffer to avoid gaps in data, but what can we do when the buffer is full? With a conventional buffer the only options are to either stop buffering new data or to overwrite the older data. This may work for some kinds of data, but if we have some elements which we know are more interesting than others then we can try to overwrite the least interesting ones - and therefore continue to buffer data, knowing that when we get the connection back, we will still be able to transmit it. 

Another application could be high-density debugging information on a computer - such as memory / cpu / networking usage per process. Typically this is too much data to store without knowing ahead of time what you are looking for - but if it could occupy a fixed amount of memory and just keep the datapoints necessary to draw a chart of usage, it might be possible to store this without much configuration.

## Implementation

This can be implemented by a priority queue combined with a doubly-linked list (or perhaps severeral linked lists), once the allocated memory for the structure has been taken, the lowest priority element is removed and its memory is used for the next element to be enqueued.

An enqueue operation when not full would have time complexity of `O(1)` for the list append and `O(log n)` for the priority queue insertion. 
An enqueue operation when full would have additional complexity of `O(log n)` for the removal of the minimum priority element.
A dequeue operation would have time complexity of `O(1)` for the list removal and `O(log n)` for the priority queue removal. 

Additionally, multiple series of data can share the same memory in the structure as long as their priorities are comparable.

## References

[Compression of Time Series by Extracting Major Extrema - Fink, Gandhi](http://www.cs.cmu.edu/~eugene/research/full/compress-series.pdf)
[Rust collections::linked_list](https://doc.rust-lang.org/src/collections/linked_list.rs.html)

