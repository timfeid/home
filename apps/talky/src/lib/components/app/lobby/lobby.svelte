<script lang="ts">
	import type { Procedures } from '../../../../../../talky-api/bindings';
	import { client, websocketClient } from '../../../client';
	import type { Unsubscribable } from '../../../rspc';
	import { onDestroy, onMount } from 'svelte';
	import { Client } from '@rspc/client';
	import { user } from '../../../user.svelte';
	import Button from '../../ui/button/button.svelte';
	import Layout from './layout.svelte';
	import type { LobbyData, LobbyPresenceData } from './lobby';

	export let joinCode: string;

	let presence: LobbyPresenceData | undefined = undefined;
	let lobby: LobbyData | undefined = undefined;
	let unsubscribe: (() => void) | undefined;

	async function pong(socketId: string) {
		const response = await client.lobby_pong.mutate({ join_code: joinCode, socket_id: socketId });
		if (response.status !== 'ok') {
			console.error('Pong failed');
			return;
		}
	}

	function onData(data: Procedures['lobby_subscribe']['output']) {
		console.log('[Lobby] Received data:', data);
		if ('Ping' in data) {
			pong(data.Ping);
		} else {
			presence = data.Data;
		}
	}

	async function getLobby(joinCode: string) {
		presence = undefined;
		if (unsubscribe) {
			unsubscribe();
		}
		try {
			if (websocketClient instanceof Client && user.accessToken) {
				unsubscribe = websocketClient.addSubscription(
					['lobby_subscribe', { join_code: joinCode, access_token: user.accessToken }],
					{
						onData,
					},
				);
			}
		} catch (e) {
			console.error('[Lobby] Error subscribing to lobby:', e);
		}
	}

	let peerConnection: RTCPeerConnection | null = null;
	let signalingChannel: WebSocket | null = null;
	let audioElement: HTMLAudioElement;

	function setupSocket() {
		let joinCode = 'room1';
		console.log('[Signaling] Connecting to ws://localhost:8080/soundhouse ...');
		signalingChannel = new WebSocket('ws://localhost:8080/soundhouse');

		signalingChannel.onopen = () => {
			const initMsg = {
				join_code: joinCode,
				auth_code: user.accessToken,
				role: 'answerer',
				type: 'init',
			};
			signalingChannel?.send(JSON.stringify(initMsg));
			console.log('[Signaling] Sent init message:', initMsg);
		};

		signalingChannel.onmessage = async (event) => {
			console.log('[Signaling] Received message:', event.data);
			let message;
			try {
				message = JSON.parse(event.data);
			} catch (error) {
				console.error('[Signaling] Error parsing message:', error);
				return;
			}

			if (!peerConnection) {
				console.error('[Signaling] No peer connection available.');
				return;
			}

			if (message.offer) {
				console.log('[Signaling] Received offer SDP:', message.offer);
				try {
					const remoteDesc = new RTCSessionDescription({ type: 'offer', sdp: message.offer });
					await peerConnection.setRemoteDescription(remoteDesc);
					console.log('[WebRTC] Remote description set from offer.');
					const answer = await peerConnection.createAnswer();
					await peerConnection.setLocalDescription(answer);
					console.log('[WebRTC] Created answer:', answer.sdp);
					const answerMsg = { type: 'answer', answer: answer.sdp, join_code: joinCode };
					signalingChannel?.send(JSON.stringify(answerMsg));
					console.log('[Signaling] Sent SDP answer:', answerMsg);
				} catch (error) {
					console.error('[Signaling] Error processing offer:', error);
				}
			} else if (message.candidate) {
				console.log('[Signaling] Received ICE Candidate:', message.candidate);
				try {
					if (
						message.candidate.candidate &&
						message.candidate.sdpMid != null &&
						message.candidate.sdpMLineIndex != null
					) {
						const iceCandidate = new RTCIceCandidate(message.candidate);
						await peerConnection.addIceCandidate(iceCandidate);
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
			} else {
				console.log('[Signaling] Unrecognized message type:', message);
			}
		};

		signalingChannel.onerror = (error) => {
			console.error('[Signaling] WebSocket error:', error);
		};

		signalingChannel.onclose = () => {
			console.log('[Signaling] Signaling channel closed.');
		};
	}

	async function startWebRTC() {
		console.log('[WebRTC] Starting as answerer...');
		peerConnection = new RTCPeerConnection({
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

		peerConnection.onicecandidate = (event) => {
			if (event.candidate && signalingChannel) {
				console.log('[WebRTC] New ICE candidate:', event.candidate);
				signalingChannel.send(JSON.stringify({ type: 'candidate', candidate: event.candidate }));
			}
		};

		peerConnection.onconnectionstatechange = () => {
			console.log(`[WebRTC] Connection State: ${peerConnection?.connectionState}`);
		};

		peerConnection.onsignalingstatechange = () => {
			console.log(`[WebRTC] Signaling State: ${peerConnection?.signalingState}`);
		};

		peerConnection.ontrack = (event) => {
			console.log('[WebRTC] Received remote track event:', event);
			let stream: MediaStream;
			if (event.streams && event.streams[0]) {
				stream = event.streams[0];
			} else {
				stream = new MediaStream([event.track]);
			}
			console.log('[WebRTC] Received stream:', stream);
			console.log('[WebRTC] Audio tracks in stream:', stream.getAudioTracks());

			console.log(stream);
			audioElement.srcObject = stream;
			audioElement.volume = 1.0;
			audioElement.muted = false;

			audioElement
				.play()
				.then(() => {
					console.log('[WebRTC] Audio playback started successfully.');

					const audioCtx = new AudioContext();
					const source = audioCtx.createMediaStreamSource(stream);
					const analyser = audioCtx.createAnalyser();

					source.connect(analyser);
					analyser.fftSize = 2048;
					const bufferLength = analyser.frequencyBinCount;
					const dataArray = new Uint8Array(bufferLength);
					function draw() {
						analyser.getByteFrequencyData(dataArray);

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

		setupSocket();

		console.log('[WebRTC] Waiting for remote offer from the test client...');
	}

	function reportStats() {
		peerConnection!.getStats(null).then((stats) => {
			const allStats = [];
			stats.forEach((report) => {
				allStats.push(report);
			});
			console.log(JSON.stringify(allStats));
		});
		setTimeout(reportStats, 1000);
	}

	function stopWebRTC() {
		console.log('[WebRTC] Stopping WebRTC connection...');
		if (peerConnection) {
			peerConnection.close();
			peerConnection = null;
		}
		if (audioElement) {
			audioElement.srcObject = null;
		}
		if (signalingChannel) {
			signalingChannel.close();
			signalingChannel = null;
		}
	}

	onMount(() => {
		getLobby(joinCode);
		doSomething();
	});

	onDestroy(() => {
		if (unsubscribe) {
			unsubscribe();
		}
		stopWebRTC();
	});

	$: if (joinCode) {
		getLobby(joinCode);
	}

	async function doSomething() {
		const response = await client.lobby_join.mutate(joinCode);
		if (response.status !== 'ok') {
			console.error('[Lobby] lobby_join failed:', response);
			return;
		}
		lobby = response.data;
		console.log('[Lobby] Joined lobby:', response);
	}
</script>

<!-- UI -->
<button on:click={startWebRTC}>Start Audio Playback</button>
<button on:click={stopWebRTC}>Stop Audio Playback</button>

<audio bind:this={audioElement} controls></audio>

{#if lobby}
	<Layout {lobby} {presence} />
{/if}

<p>{joinCode}</p>
<Button onclick={doSomething} />
<pre>{JSON.stringify(presence, null, 2)}</pre>
