<script lang="ts">
	import { page } from '$app/state';
	import Channel from '$lib/components/channel/channel.svelte';
	import Niche from '$lib/components/niche/niche.svelte';
	import ChannelsSidebar from '$lib/layout/channels-sidebar.svelte';
	import Layout from '$lib/layout/layout.svelte';
</script>

<Layout>
	<ChannelsSidebar />

	<div class="flex flex-1 flex-col">
		{#key page.params.niche}
			<Niche slug={page.params.niche}>
				{#snippet withNiche({ niche })}
					{#key page.params.channel}
						<Channel {niche} slug={page.params.channel} />
					{/key}
				{/snippet}
			</Niche>
		{/key}
	</div>
</Layout>
