import { getContext } from 'svelte';
import { user } from './user.svelte.js';
import { connectAudio, isTauri } from './tauri/tauri.js';
import { page } from '$app/state';
import { type ClientInfoMsg, type IncomingMessage, type RoomResource } from '@talky/soundhouse';
import { env } from '$env/dynamic/public';


const MAX_RETRIES = 5;
const RETRY_DELAY_MS = 3000;

class EventEmitter {
  private events: { [key: string]: CallableFunction[] } = {};

  on(event: string, listener: CallableFunction) {
    if (!this.events[event]) {
      this.events[event] = [];
    }
    this.events[event].push(listener);
  }

  off(event: string, listener: CallableFunction) {
    if (!this.events[event]) return;
    this.events[event] = this.events[event].filter((l) => l !== listener);
  }

  emit(event: string, ...args: unknown[]) {
    if (!this.events[event]) return;
    this.events[event].forEach((listener) => listener(...args));
  }
}

export class Presence extends EventEmitter {
  socket = $state<WebSocket | null>(null);
  channelConnection = $state({
    status: 'init',
    id: '',
  });

  status = $state<
    'idle' | 'connecting' | 'connected' | 'handshake' | 'reconnecting' | 'closed' | 'error'
  >('idle');
  error = $state<Event | Error | null>(null);
  retryCount = $state(0);
  private retryTimer = $state<ReturnType<typeof setTimeout> | null>(null);
  private explicitlyClosed = $state(false);

  peerConnection = $state<RTCPeerConnection | null>(null);
  lastMessage = $state<IncomingMessage | null>(null);
  currentNicheId = $state('');
  activeClients = $state<ClientInfoMsg[]>([]);
  activeChannels = $state<
    Partial<{
      [x: string]: RoomResource;
    }>
  >({});

  isConnected = $derived(this.status === 'connected');
  isTrying = $derived(this.status === 'connecting' || this.status === 'reconnecting');

  connectedChannel = $derived(
    this.channelConnection.status === 'connected' ? this.channelConnection : undefined,
  );

  setup() {
    $effect(() => {
      const token = user.accessToken;

      console.debug(
        `[Presence Effect Run] Token: ${token ? 'present' : 'absent'}, Status: ${this.status}, ExplicitlyClosed: ${this.explicitlyClosed}`,
      );

      if (token && this.status === 'idle') {
        console.log(
          '[Presence Effect] Token appeared and status is idle. Initiating connection process.',
        );
        this.explicitlyClosed = false;
        this.retryCount = 0;
        this.attemptConnection(token);
      } else if (!token && this.status !== 'idle') {
        console.log(
          '[Presence Effect] Token disappeared. Disconnecting and setting status to idle.',
        );
        this.disconnect(true);
        this.status = 'idle';
      }
    });
  }

  async connected() {
    console.log('joined channel, lets attempt to join voice now');
    this.createPeerConnection();
    if (isTauri && user.accessToken && this.connectedChannel) {
      connectAudio(user.accessToken, this.connectedChannel.id, page.params.niche || 'devils');
    }
  }

  joinChannel(channelId: string) {
    this.channelConnection = {
      status: 'init',
      id: channelId,
    };

    this.sendMessage({
      type: 'join',
      role: 'answerer',
      channel_id: channelId,
    });
  }

  private attemptConnection(token: string | undefined): void {
    if (!token) {
      console.warn('[AttemptConnection] Aborted: No token provided.');
      if (this.status !== 'idle') this.status = 'idle';
      return;
    }

    if (
      this.socket ||
      this.status === 'handshake' ||
      this.status === 'connected' ||
      this.status === 'connecting'
    ) {
      console.warn(
        `[AttemptConnection] Aborted: Already connected or attempting (Status: ${this.status}).`,
      );
      return;
    }

    if (this.explicitlyClosed) {
      console.log('[AttemptConnection] Aborted: Connection was explicitly closed earlier.');
      if (this.status !== 'closed' && this.status !== 'idle') this.status = 'closed';
      return;
    }

    if (this.retryCount >= MAX_RETRIES) {
      console.error(
        `[AttemptConnection] Aborted: Max retries (${MAX_RETRIES}) were previously reached.`,
      );
      if (this.status !== 'error') this.status = 'error';
      this.error = this.error || new Error(`Max retries (${MAX_RETRIES}) reached.`);
      return;
    }

    const isInitialAttempt = this.retryCount === 0;
    console.log(
      `[AttemptConnection] Attempting WebSocket connection (Attempt: ${this.retryCount + 1}/${MAX_RETRIES})...`,
    );
    this.status = isInitialAttempt ? 'connecting' : 'reconnecting';
    this.error = null;
    this.clearRetryTimer();

    try {
      const ws = new WebSocket(env.PUBLIC_SOUNDHOUSE_URL);
      this.socket = ws;

      ws.onopen = this.handleOpen.bind(this, ws, token);
      ws.onmessage = this.handleMessage.bind(this, ws);
      ws.onerror = this.handleError.bind(this, ws);
      ws.onclose = this.handleClose.bind(this, ws);
    } catch (err) {
      console.error('[AttemptConnection] Error creating WebSocket instance:', err);
      this.socket = null;
      this.status = 'error';
      this.error = err instanceof Error ? err : new Error('WebSocket creation failed');
    }
  }

  private handleOpen(ws: WebSocket, token: string): void {
    if (this.socket !== ws) {
      console.warn("[HandleOpen] Ignoring 'open' event for stale socket.");
      return;
    }
    console.log('[Presence] WebSocket connection established.');
    this.status = 'handshake';
    this.retryCount = 0;
    this.explicitlyClosed = false;
    this.clearRetryTimer();

    const initMsg = {
      type: 'init',
      auth_code: token,
      role: 'presence',
    };
    try {
      this.emit('connectionOpen', initMsg);
      console.log('[Presence] Sending init message');
      ws.send(JSON.stringify(initMsg));
    } catch (err) {
      console.error('[Presence] Failed to send init message immediately after open:', err);
      this.status = 'error';
      this.error = err instanceof Error ? err : new Error('Failed to send init message');
      this.explicitlyClosed = true;
      ws.close(1011, 'Init send failed');
    }

    console.log('[Presence] Handshake complete');
    this.status = 'connected';
  }

  private createPeerConnection() {
    console.log('[WebRTC] Starting as answerer...');
    this.peerConnection = new RTCPeerConnection({
      iceServers: [
        {
          urls: [
            'stun:server.loc:31899',
            'turn:server.loc:30665?transport=udp',
            'turn:server.loc:31953?transport=tcp',
          ],
          username: 'coturn',
          credential: 'password',
        },
      ],
      iceTransportPolicy: 'all',
    });

    this.peerConnection.onicecandidate = (event) => {
      if (event.candidate && this.socket) {
        console.log('[WebRTC] New ICE candidate:', event.candidate);
        this.socket.send(
          JSON.stringify({
            type: 'candidate',
            candidate: event.candidate,
            channel_id: this.connectedChannel!.id,
            niche_id: page.params.niche,
          }),
        );
      }
    };

    // Add this to monitor connection state
    this.peerConnection.onconnectionstatechange = () => {
      console.log(`[WebRTC] Connection State: ${this.peerConnection?.connectionState}`);
    };

    // Add this to monitor soundhouse state
    this.peerConnection.onsignalingstatechange = () => {
      console.log(`[WebRTC] Signaling State: ${this.peerConnection?.signalingState}`);
    };

    // When remote media is received, attach it to the audio element.
    this.peerConnection.ontrack = (event) => {
      console.log('[WebRTC] Received remote track event:', event);
      let stream: MediaStream;
      if (event.streams && event.streams[0]) {
        stream = event.streams[0];
      } else {
        stream = new MediaStream([event.track]);
      }
      console.log('[WebRTC] Received stream:', stream);
      console.log('[WebRTC] Audio tracks in stream:', stream.getAudioTracks());

      // Attach stream to the audio element.
      console.log(stream);
      const audioElement = document.createElement('audio');
      document.body.appendChild(audioElement);
      audioElement.srcObject = stream;
      audioElement.volume = 1.0;
      audioElement.muted = false; // ensure not muted

      audioElement
        .play()
        .then(() => {
          console.log('[WebRTC] Audio playback started successfully.');
          // Use Web Audio API to analyze the stream.
          const audioCtx = new AudioContext();
          const source = audioCtx.createMediaStreamSource(stream);
          const analyser = audioCtx.createAnalyser();

          source.connect(analyser);
          analyser.fftSize = 2048;
          const bufferLength = analyser.frequencyBinCount;
          const dataArray = new Uint8Array(bufferLength);
          function draw() {
            analyser.getByteFrequencyData(dataArray);
            // Log average volume (or analyze the data in another way)
            const avg = dataArray.reduce((sum, value) => sum + value, 0) / dataArray.length;
            console.log('[Audio Analyser] Average volume:', avg);
            requestAnimationFrame(draw);
          }
          draw();
        })
        .catch((error) => {
          console.error('[WebRTC] Error starting audio playback:', error);
        });
    };

    // Set up the soundhouse channel.
    // this.reportStats();
    console.log('[WebRTC] Waiting for remote offer from the test client...');
  }

  reportStats() {
    this.peerConnection!.getStats(null).then((stats) => {
      const allStats = [];
      stats.forEach((report) => {
        allStats.push(report);
      });
      console.log(JSON.stringify(allStats));
    });
    setTimeout(this.reportStats.bind(this), 1000);
  }

  private candidate(message: { candidate: RTCIceCandidate }) {
    console.log('[Signaling] Received ICE Candidate:', message.candidate);
    if (!this.peerConnection) {
      return;
    }
    try {
      // Validate that candidate fields exist before constructing the RTCIceCandidate.
      if (
        message.candidate.candidate &&
        message.candidate.sdpMid != null &&
        message.candidate.sdpMLineIndex != null
      ) {
        const iceCandidate = new RTCIceCandidate(message.candidate);
        this.peerConnection.addIceCandidate(iceCandidate);
        console.log('[WebRTC] Added ICE candidate successfully.');
      } else {
        console.error(
          '[Signaling] Received candidate is missing required fields:',
          message.candidate,
        );
      }
    } catch (error) {
      console.error('[WebRTC] Error adding ICE candidate:', error);
    }
  }

  private async offer(message: IncomingMessage) {
    if (message.type !== 'offer') {
      console.warn('[Signaling] Incorrect message type. Expected "offer".');
      return;
    }
    console.log('[Signaling] Received offer SDP:', message.offer);
    if (!this.peerConnection) {
      return;
    }
    try {
      const remoteDesc = new RTCSessionDescription({ type: 'offer', sdp: message.offer });
      await this.peerConnection.setRemoteDescription(remoteDesc);
      console.log('[WebRTC] Remote description set from offer.');
      const answer = await this.peerConnection.createAnswer();
      await this.peerConnection.setLocalDescription(answer);
      console.log('[WebRTC] Created answer:', answer.sdp);
      const answerMsg = {
        type: 'answer',
        answer: answer.sdp,
        channel_id: this.connectedChannel!.id,
        niche_id: page.params.niche,
      };
      this.socket?.send(JSON.stringify(answerMsg));
      console.log('[Signaling] Sent SDP answer:', answerMsg);
    } catch (error) {
      console.error('[Signaling] Error processing offer:', error);
    }
  }
  private handleMessage(ws: WebSocket, event: MessageEvent): void {
    if (this.socket !== ws) return;

    try {
      const message = JSON.parse(event.data) as IncomingMessage;
      this.lastMessage = message;

      switch (message.type) {
        case 'active_channels':
          this.activeChannels = message.channels;

          this.emit('[Presence] Updated active channels', this.activeChannels);
          break;
        case 'candidate':
          this.candidate(message as unknown as { candidate: RTCIceCandidate });
          break;
        case 'offer':
          this.offer(message);
          break;
        case 'active_clients_update':
          this.activeClients = message.clients;
          this.emit('activeClientsUpdated', this.activeClients);
          console.log('[Presence] Updated active clients:', this.activeClients.length);
          break;
        case 'error':
          console.error('[Presence] Received server error message:', message.message);
          this.emit('serverError', message.message);
          break;
        case 'chat_message_broadcast':
          window.dispatchEvent(new CustomEvent('chat', { detail: message }));
          this.emit('chatMessageReceived', message);
          break;

        default:
          console.warn('[Presence] Received unhandled message :', message);
          this.emit('unhandledMessage', message);
      }
    } catch (error) {
      console.error('[Presence] Error parsing received message:', error, 'Raw data:', event.data);
      this.emit('messageParsingError', error, event.data);
    }
  }

  private handleError(ws: WebSocket, event: Event): void {
    if (this.socket !== ws) return;

    console.error('[Presence] WebSocket error event:', event);
    this.error = event;
    this.emit('socketError', event);
  }

  private handleClose(ws: WebSocket, event: CloseEvent): void {
    if (this.socket !== ws) {
      console.log('[Presence] Ignoring close event for non-active/stale socket.');
      return;
    }

    console.log(
      `[Presence] WebSocket connection closed. Code: ${event.code}, Reason: "${event.reason}", Clean: ${event.wasClean}, ExplicitlyClosed: ${this.explicitlyClosed}`,
    );
    this.socket = null;
    this.emit('connectionClosed', event);
    this.clearRetryTimer();

    const shouldAttemptRetry =
      user.accessToken &&
      !this.explicitlyClosed &&
      this.retryCount < MAX_RETRIES &&
      event.code !== 1000;

    if (shouldAttemptRetry) {
      this.retryCount++;
      console.log(
        `[Presence] Connection closed unexpectedly. Scheduling retry ${this.retryCount}/${MAX_RETRIES} in ${RETRY_DELAY_MS}ms...`,
      );
      this.status = 'reconnecting';
      this.retryTimer = setTimeout(() => {
        this.retryTimer = null;

        if (user.accessToken && !this.explicitlyClosed) {
          this.attemptConnection(user.accessToken);
        } else {
          console.log(
            '[Presence] Retry aborted inside timeout: Token missing or explicitly closed during delay.',
          );
          this.status = user.accessToken ? 'closed' : 'idle';
          if (!user.accessToken) this.retryCount = 0;
        }
      }, RETRY_DELAY_MS);
    } else {
      console.log('[Presence] No retry condition met upon close.');
      if (this.retryCount >= MAX_RETRIES && !this.explicitlyClosed && user.accessToken) {
        console.error(`[Presence] Max retries (${MAX_RETRIES}) reached. Setting status to error.`);
        this.status = 'error';
        this.error = this.error || new Error(`Connection failed after ${MAX_RETRIES} retries.`);
        this.emit('maxRetriesExceeded');
      } else if (!user.accessToken) {
        console.log('[Presence] User logged out. Setting status to idle.');
        this.status = 'idle';
        this.retryCount = 0;
        this.emit('userLoggedOut');
      } else {
        console.log(
          `[Presence] Setting status to closed (Explicit: ${this.explicitlyClosed}, Code: ${event.code}).`,
        );
        this.status = 'closed';

        if (this.explicitlyClosed || event.code === 1000) {
          this.retryCount = 0;
        }
      }
      this.clearRetryTimer();
    }
  }

  private clearRetryTimer(): void {
    if (this.retryTimer) {
      clearTimeout(this.retryTimer);
      this.retryTimer = null;
    }
  }

  private cleanupSocketInstance(ws: WebSocket | null): void {
    if (ws) {
      ws.onopen = null;
      ws.onmessage = null;
      ws.onerror = null;
      ws.onclose = null;
      console.debug('[Presence] Cleaned up listeners for socket instance.');
    }
  }

  /** Performs cleanup: clears timers, removes listeners, closes socket if open. */
  cleanup(): void {
    console.log('[Presence Cleanup] Performing cleanup...');
    this.clearRetryTimer();
    const ws = this.socket;
    if (ws) {
      this.cleanupSocketInstance(ws);

      if (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING) {
        console.log('[Presence Cleanup] Closing active WebSocket connection...');
        this.explicitlyClosed = true;
        ws.close(1000, 'Client cleanup');
      }
      this.socket = null;
    }

    if (!user.accessToken) {
      this.status = 'idle';
      this.retryCount = 0;
    } else if (!this.socket) {
      if (this.status !== 'error' && this.status !== 'reconnecting') {
        this.status = 'closed';
      }
    }
  }

  /** Disconnects the WebSocket manually, preventing automatic retries. */
  disconnect(isLogout: boolean = false): void {
    console.log(`[Presence Disconnect] Manual disconnect called (isLogout: ${isLogout}).`);
    this.explicitlyClosed = true;
    this.clearRetryTimer();

    const ws = this.socket;
    if (ws) {
      this.cleanupSocketInstance(ws);

      if (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING) {
        console.log('[Presence Disconnect] Closing active WebSocket connection...');
        ws.close(1000, isLogout ? 'User logged out' : 'Manual disconnect');
      } else {
        this.socket = null;
        this.status = isLogout ? 'idle' : 'closed';
      }
    } else {
      console.log('[Presence Disconnect] No active socket to close.');
      this.status = isLogout ? 'idle' : 'closed';
    }

    this.retryCount = 0;
  }

  sendMessage(message: object): boolean {
    if (!this.socket || this.status !== 'connected') {
      console.error(
        `[Presence SendMessage] Cannot send message, WebSocket status is not 'open' (Status: ${this.status}).`,
      );
      return false;
    }
    try {
      const jsonMessage = JSON.stringify(message);
      this.socket.send(jsonMessage);
      console.debug('[Presence SendMessage] Sent message:', message);
      this.emit('messageSent', message);
      return true;
    } catch (error) {
      console.error('[Presence SendMessage] Error sending message:', error);
      this.status = 'error';
      this.error = error instanceof Error ? error : new Error('WebSocket send failed');
      this.explicitlyClosed = true;
      this.socket?.close(1011, 'Send message failed');
      this.emit('sendMessageFailed', error);
      return false;
    }
  }
}

export function withPresence() {
  return getContext('presence') as Presence;
}
