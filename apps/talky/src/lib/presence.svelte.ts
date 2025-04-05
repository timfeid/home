import { getContext } from 'svelte';
import { user } from './user.svelte.js';

interface ClientInfoMsg {
	id: string;
	user_id: string;
}

interface ActiveClientsMessage {
	type: 'active_clients_update';
	clients: ClientInfoMsg[];
}

interface ErrorMessage {
	type: 'error';
	message: string;
}

export interface ChatMessage {
	type: 'chat_message_broadcast';
	message: string;
	content: string;
	channel_id: string;
	user_id: string;
	timestamp: string;
}

export type IncomingServerMessage = ActiveClientsMessage | ErrorMessage | ChatMessage;

const SOUNDHOUSE_URL = 'ws://localhost:8080/soundhouse';
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

	status = $state<'idle' | 'connecting' | 'open' | 'reconnecting' | 'closed' | 'error'>('idle');
	error = $state<Event | Error | null>(null);
	retryCount = $state(0);
	private retryTimer = $state<ReturnType<typeof setTimeout> | null>(null);
	private explicitlyClosed = $state(false);

	lastMessage = $state<IncomingServerMessage | null>(null);
	activeClients = $state<ClientInfoMsg[]>([]);

	isConnected = $derived(this.status === 'open');
	isTrying = $derived(this.status === 'connecting' || this.status === 'reconnecting');

	setup() {
		$effect(() => {
			const token = user.accessToken;

			console.debug(
				`[Presence Effect Run] Token: ${token ? 'present' : 'absent'}, Status: ${this.status}, ExplicitlyClosed: ${this.explicitlyClosed}`
			);

			if (token && this.status === 'idle') {
				console.log(
					'[Presence Effect] Token appeared and status is idle. Initiating connection process.'
				);
				this.explicitlyClosed = false;
				this.retryCount = 0;
				this.attemptConnection(token);
			} else if (!token && this.status !== 'idle') {
				console.log(
					'[Presence Effect] Token disappeared. Disconnecting and setting status to idle.'
				);
				this.disconnect(true);
				this.status = 'idle';
			}
		});
	}

	private attemptConnection(token: string | undefined): void {
		if (!token) {
			console.warn('[AttemptConnection] Aborted: No token provided.');
			if (this.status !== 'idle') this.status = 'idle';
			return;
		}

		if (this.socket || this.status === 'open' || this.status === 'connecting') {
			console.warn(
				`[AttemptConnection] Aborted: Already connected or attempting (Status: ${this.status}).`
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
				`[AttemptConnection] Aborted: Max retries (${MAX_RETRIES}) were previously reached.`
			);
			if (this.status !== 'error') this.status = 'error';
			this.error = this.error || new Error(`Max retries (${MAX_RETRIES}) reached.`);
			return;
		}

		const isInitialAttempt = this.retryCount === 0;
		console.log(
			`[AttemptConnection] Attempting WebSocket connection (Attempt: ${this.retryCount + 1}/${MAX_RETRIES})...`
		);
		this.status = isInitialAttempt ? 'connecting' : 'reconnecting';
		this.error = null;
		this.clearRetryTimer();

		try {
			const ws = new WebSocket(SOUNDHOUSE_URL);
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
		this.status = 'open';
		this.retryCount = 0;
		this.explicitlyClosed = false;
		this.clearRetryTimer();

		const initMsg = {
			type: 'init',
			auth_code: token,
			role: 'presence'
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
	}

	private handleMessage(ws: WebSocket, event: MessageEvent): void {
		if (this.socket !== ws) return;

		try {
			const message = JSON.parse(event.data) as IncomingServerMessage;
			this.lastMessage = message;

			switch (message.type) {
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
			`[Presence] WebSocket connection closed. Code: ${event.code}, Reason: "${event.reason}", Clean: ${event.wasClean}, ExplicitlyClosed: ${this.explicitlyClosed}`
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
				`[Presence] Connection closed unexpectedly. Scheduling retry ${this.retryCount}/${MAX_RETRIES} in ${RETRY_DELAY_MS}ms...`
			);
			this.status = 'reconnecting';
			this.retryTimer = setTimeout(() => {
				this.retryTimer = null;

				if (user.accessToken && !this.explicitlyClosed) {
					this.attemptConnection(user.accessToken);
				} else {
					console.log(
						'[Presence] Retry aborted inside timeout: Token missing or explicitly closed during delay.'
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
					`[Presence] Setting status to closed (Explicit: ${this.explicitlyClosed}, Code: ${event.code}).`
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

	async sendMessage(message: object): Promise<boolean> {
		if (!this.socket || this.status !== 'open') {
			console.error(
				`[Presence SendMessage] Cannot send message, WebSocket status is not 'open' (Status: ${this.status}).`
			);
			return false;
		}
		try {
			const jsonMessage = JSON.stringify(message);
			await this.socket.send(jsonMessage);
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
