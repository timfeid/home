<script lang="ts">
	import { onMount } from 'svelte';
	import ServerSidebar from '../layout/server-sidebar.svelte';
	import ChannelSidebar from '../layout/channel-sidebar.svelte';
	import ChatArea from '../layout/chat-area.svelte';
	import MembersList from '../layout/members-list.svelte';
	import MobileNav from '../layout/mobile-nav.svelte';
	import type { LobbyData, LobbyPresenceData } from './lobby';

	let { lobby, presence }: { lobby: LobbyData; presence?: LobbyPresenceData } = $props();

	// Local state variables
	let activeServer = $state(0);
	let activeChannel = $state(0);
	let showChannels = $state(true);
	let showMembers = $state(true);

	// Media query booleans
	let isMobile = $state(false);
	let isTablet = $state(false);

	let mobileQuery: MediaQueryList;
	let tabletQuery: MediaQueryList;

	function handleMobileQuery(e: MediaQueryListEvent) {
		isMobile = e.matches;
	}

	function handleTabletQuery(e: MediaQueryListEvent) {
		isTablet = e.matches;
	}

	// Set up media queries on mount
	onMount(() => {
		mobileQuery = window.matchMedia('(max-width: 768px)');
		tabletQuery = window.matchMedia('(max-width: 1024px)');

		// Set initial values
		isMobile = mobileQuery.matches;
		isTablet = tabletQuery.matches;

		mobileQuery.addEventListener('change', handleMobileQuery);
		tabletQuery.addEventListener('change', handleTabletQuery);

		return () => {
			mobileQuery.removeEventListener('change', handleMobileQuery);
			tabletQuery.removeEventListener('change', handleTabletQuery);
		};
	});

	// Reactively update sidebar visibility based on viewport size
	$effect(() => {
		if (isMobile) {
			showChannels = false;
			showMembers = false;
		} else if (isTablet) {
			showChannels = true;
			showMembers = false;
		} else {
			showChannels = true;
			showMembers = true;
		}
	});

	// Helper functions for updating state (passed as props to child components)
	function handleSetActiveServer(server: number) {
		activeServer = server;
	}

	function handleSetActiveChannel(channel: number) {
		activeChannel = channel;
	}

	function closeChannels() {
		showChannels = false;
	}

	function closeMembers() {
		showMembers = false;
	}
</script>

<div class="flex h-screen overflow-hidden bg-[#36393f] text-gray-100">
	<!-- Server Sidebar -->
	<ServerSidebar {activeServer} setActiveServer={handleSetActiveServer} />

	<!-- Channel Sidebar (conditionally rendered) -->
	{#if showChannels || !isMobile}
		<ChannelSidebar
			{activeChannel}
			setActiveChannel={handleSetActiveChannel}
			onClose={closeChannels}
		/>
	{/if}

	<!-- Main content area -->
	<div class="flex min-w-0 flex-1 flex-col">
		{#if isMobile}
			<MobileNav
				{showChannels}
				setShowChannels={(value) => (showChannels = value)}
				{showMembers}
				setShowMembers={(value) => (showMembers = value)}
			/>
		{/if}

		<div class="flex flex-1 overflow-hidden">
			<ChatArea {activeServer} {activeChannel} />

			{#if showMembers}
				<MembersList {lobby} onClose={closeMembers} />
			{/if}
		</div>
	</div>
</div>
