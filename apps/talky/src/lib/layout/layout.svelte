<script lang="ts">
	import ChannelList from '$lib/channel-list/channel-list.svelte';
	import ChatArea from '$lib/chat-area/chat-area.svelte';
	import ServerSelector from '$lib/talky-server-selector/talky-server-selector.svelte';
	import ServerSettings from '$lib/talky-server-settings/talky-server-settings.svelte';
	import StatusBar from '$lib/status-bar/status-bar.svelte';
	import UserList from '$lib/user-list/user-list.svelte';
	import VoiceControls from '$lib/voice-controls/voice-controls.svelte';

	let showSettings = $state(false);
</script>

<div class="flex h-screen bg-gray-100 text-gray-900">
	<!-- Main Content Area -->
	<div class="flex flex-grow flex-col">
		<!-- Top Bar -->
		<div class="border-b border-gray-400 bg-gray-200 p-1">
			<ServerSelector />
		</div>

		<!-- Main Content -->
		<div class="flex flex-grow overflow-hidden">
			<!-- Channel List -->
			<div class="flex w-48 flex-col border-r border-gray-400 bg-gray-200">
				<ChannelList />
			</div>

			<!-- Chat Area -->
			<div class="flex flex-grow flex-col">
				<ChatArea />
			</div>

			<!-- User List -->
			<div class="flex w-40 flex-col border-l border-gray-400 bg-gray-200">
				<UserList />
			</div>
		</div>

		<!-- Voice Controls -->
		<VoiceControls />

		<!-- Status Bar -->
		<StatusBar />
	</div>

	<!-- Server Settings Modal -->
	{#if showSettings}
		<ServerSettings onClose={() => (showSettings = false)} />
	{/if}
</div>

<!-- Open settings button for demo purposes -->
<button
	class="fixed bottom-4 right-4 rounded-full bg-blue-600 p-2 text-white"
	onclick={() => (showSettings = true)}
>
	Settings
</button>
