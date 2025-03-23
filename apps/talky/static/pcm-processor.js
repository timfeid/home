class RingBuffer {
	constructor(capacity) {
		this.capacity = capacity;
		this.buffer = new Float32Array(capacity);
		this.writeIndex = 0;
		this.readIndex = 0;
		this.size = 0;
	}

	// Push new data into the ring buffer.
	push(data) {
		for (let i = 0; i < data.length; i++) {
			this.buffer[this.writeIndex] = data[i];
			this.writeIndex = (this.writeIndex + 1) % this.capacity;
			if (this.size < this.capacity) {
				this.size++;
			} else {
				// If the buffer is full, we overwrite the oldest data.
				this.readIndex = (this.readIndex + 1) % this.capacity;
			}
		}
	}

	// Pop 'numSamples' from the buffer.
	pop(numSamples) {
		if (this.size < numSamples) {
			return null; // Not enough data available.
		}
		const result = new Float32Array(numSamples);
		for (let i = 0; i < numSamples; i++) {
			result[i] = this.buffer[(this.readIndex + i) % this.capacity];
		}
		this.readIndex = (this.readIndex + numSamples) % this.capacity;
		this.size -= numSamples;
		return result;
	}
}

class PCMProcessor extends AudioWorkletProcessor {
	constructor() {
		super();
		// Create a ring buffer with capacity for 2 seconds of audio.
		// For mono audio at 48000 Hz, that's 96000 samples.
		this.ringBuffer = new RingBuffer(800000);
		this.port.onmessage = (event) => {
			if (event.data) {
				const incoming = new Float32Array(event.data);
				this.ringBuffer.push(incoming);
			}
		};
	}

	process(inputs, outputs, parameters) {
		const output = outputs[0];
		// Assume mono output for this example; adjust if stereo.
		const channel = output[0];
		const framesNeeded = channel.length;

		const data = this.ringBuffer.pop(framesNeeded);
		if (data) {
			channel.set(data);
		} else {
			// If there's not enough data, output silence.
			channel.fill(0);
		}
		return true;
	}
}

registerProcessor('pcm-processor', PCMProcessor);
