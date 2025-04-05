<script lang="ts">
	import type { Snippet } from 'svelte';
	import ChannelsSidebar from './channels-sidebar.svelte';
	import StatusBar from '$lib/status-bar/status-bar.svelte';

	let { children }: { children: Snippet } = $props();
</script>

<div class="flex h-screen flex-col bg-background">
	<div class="flex h-full flex-grow">
		<ChannelsSidebar />

		<div class="flex flex-1 flex-col">
			{@render children()}
		</div>
	</div>
	<StatusBar />
</div>
