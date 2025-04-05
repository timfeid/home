<script lang="ts">
	import { page } from '$app/state';
	import { withChannelList } from '$lib/channel-list.svelte';
	import { Button } from '$lib/components/ui/button';
	import { cn } from '$lib/utils';
	import { Hash, MessageSquare } from 'lucide-svelte';
	const state = $state({
		activeChannel: ['sports', 'hockey', 'nhl', 'devils', 'news'],
		showBrowse: false
	});

	// const joinedChannels: Channel[] = [
	// 	{
	// 		id: 'devils-news',
	// 		name: 'Devils News',
	// 		path: ['sports', 'hockey', 'nhl', 'devils', 'news'],
	// 		type: 'news',
	// 		teamLogo: '/placeholder.svg?height=24&width=24',
	// 		teamName: 'Devils',
	// 		unread: 3
	// 	},
	// 	{
	// 		id: 'devils-gameday',
	// 		name: 'Devils Gameday',
	// 		path: ['sports', 'hockey', 'nhl', 'devils', 'gameday'],
	// 		type: 'chat',
	// 		teamLogo: '/placeholder.svg?height=24&width=24',
	// 		teamName: 'Devils'
	// 	},
	// 	{
	// 		id: 'devils-memes',
	// 		name: 'Devils Memes',
	// 		path: ['sports', 'hockey', 'nhl', 'devils', 'memes'],
	// 		type: 'news',
	// 		teamLogo: '/placeholder.svg?height=24&width=24',
	// 		teamName: 'Devils'
	// 	},
	// 	{
	// 		id: 'rangers-news',
	// 		name: 'Rangers News',
	// 		path: ['sports', 'hockey', 'nhl', 'rangers', 'news'],
	// 		type: 'news',
	// 		teamLogo: '/placeholder.svg?height=24&width=24',
	// 		teamName: 'Rangers',
	// 		unread: 5
	// 	},
	// 	{
	// 		id: 'javascript-help',
	// 		name: 'JavaScript Help',
	// 		path: ['tech', 'programming', 'javascript', 'help'],
	// 		type: 'chat',
	// 		teamLogo: '/placeholder.svg?height=24&width=24',
	// 		teamName: 'JS'
	// 	}
	// ];
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
					{:else}
						<Hash class="mr-1.5 h-4 w-4 flex-shrink-0" />
					{/if}
					<span class="truncate font-light">{channel.name}</span>
				</div>
			</Button>
		{/each}
	</div>
</div>
