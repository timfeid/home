<script lang="ts">
	import {
		Mic,
		MicOff,
		Volume2,
		VolumeX,
		PhoneOff,
		ChevronUp,
		ChevronDown,
		Users
	} from 'lucide-svelte';
	import { withPresence } from '$lib/presence.svelte';
	import Button from '../ui/button/button.svelte';
	import GroupedAvatars from '../ui/avatar/grouped-avatars.svelte';

	let muted = $state(false);
	let deafened = $state(false);
	let expanded = $state(true);
	const presence = withPresence();

	const connectedUsers = $derived.by(() => {
		if (!presence.channelConnection) {
			return [];
		}
		const wat = presence.activeChannels[presence.channelConnection.id]?.users || {};

		const response = Object.values(wat);
		return response.map((x) => x?.[0].user).filter((x) => !!x);
	});

	function toggleMute() {
		muted = !muted;
	}

	function toggleDeafen() {
		deafened = !deafened;
	}

	function toggleExpanded() {
		expanded = !expanded;
	}

	function onDisconnect() {}
</script>

<div class="overflow-hidden bg-sidebar-primary shadow-lg">
	<div class="flex items-center p-2">
		<div class="mr-auto flex items-center space-x-1">
			<Button onclick={toggleMute} variant="ghost" size="icon" title={muted ? 'Unmute' : 'Mute'}>
				{#if muted}
					<MicOff />
				{:else}
					<Mic />
				{/if}
			</Button>
			<Button
				onclick={toggleDeafen}
				variant="ghost"
				size="icon"
				title={deafened ? 'Undeafen' : 'Deafen'}
			>
				{#if deafened}
					<VolumeX />
				{:else}
					<Volume2 />
				{/if}
			</Button>
		</div>
		<GroupedAvatars avatars={connectedUsers} />
		<Button
			class="ml-2"
			onclick={onDisconnect}
			size="icon"
			variant="destructive"
			title="Disconnect"
		>
			<PhoneOff />
		</Button>
	</div>
</div>
