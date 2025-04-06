<script lang="ts">
	import { page } from '$app/state';
	import { withChannelList } from '$lib/channel-list.svelte';
	import { Button } from '$lib/components/ui/button';
	import { cn } from '$lib/utils';
	import { AudioLines, Hash, MessageSquare } from 'lucide-svelte';

	const channels = withChannelList();
</script>

<div class="w-72 flex-shrink-0 overflow-y-auto bg-sidebar text-sidebar-foreground">
	<div class="mb-4 space-y-[1px] p-2.5">
		{#each channels.channels as channel}
			<Button
				href="/of/devils/{channel.slug}"
				class={cn(
					'group flex h-auto w-full items-center px-2 py-1.5 text-left hover:bg-background/60',
					page.params.channel === channel.slug
						? 'bg-background text-foreground/60'
						: 'bg-transparent text-foreground/60'
				)}
			>
				<div class="flex flex-1 items-center overflow-hidden">
					{#if channel.type === 'chat'}
						<MessageSquare class="mr-1.5 h-4 w-4 flex-shrink-0" />
					{:else if channel.type === 'feed'}
						<Hash class="mr-1.5 h-4 w-4 flex-shrink-0" />
					{:else}
						<AudioLines class="mr-1.5 h-4 w-4 flex-shrink-0" />
					{/if}
					<span class="truncate font-light">{channel.name}</span>
				</div>
			</Button>
		{/each}
	</div>
</div>
