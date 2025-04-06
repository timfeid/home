<script lang="ts">
	import { page } from '$app/state';
	import Channel from '$lib/components/channel/channel.svelte';
	import Niche from '$lib/components/niche/niche.svelte';

	// let { children } = $props();
</script>

{#key page.params.niche}
	<Niche slug={page.params.niche}>
		{#snippet withNiche({ niche })}
			{#key page.params.channel}
				<Channel {niche} slug={page.params.channel} />
			{/key}
		{/snippet}
	</Niche>
{/key}
