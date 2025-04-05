<script lang="ts">
	import type { Snippet } from 'svelte';
	import StatusBar from '$lib/status-bar/status-bar.svelte';

	let { children }: { children: Snippet } = $props();
</script>

<div class="flex h-full flex-grow">
	{@render children()}
</div>
<StatusBar />
