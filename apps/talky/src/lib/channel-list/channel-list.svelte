<script lang="ts">
	import { Hash, Volume2 } from 'lucide-svelte';

	let activeChannel = $state('#gaming');

	// Mock channel data
	const channels = [
		{ id: 1, name: '#gaming', type: 'text', unread: false },
		{ id: 2, name: '#help', type: 'text', unread: true },
		{ id: 3, name: '#memes', type: 'text', unread: false },
		{ id: 4, name: '#lobby', type: 'voice', users: 3 },
		{ id: 5, name: '#team1', type: 'voice', users: 2 },
		{ id: 6, name: '#afk', type: 'voice', users: 1 }
	];
</script>

<div class="bg-blue-800 p-1 text-xs font-bold text-white">CHANNELS</div>
<div class="flex-1 overflow-y-auto">
	{#each channels as channel (channel.id)}
		<button
			class="flex w-full items-center px-2 py-1 text-left text-sm {activeChannel === channel.name
				? 'bg-blue-600 text-white'
				: 'hover:bg-gray-300'}"
			onclick={() => (activeChannel = channel.name)}
		>
			{#if channel.type === 'text'}
				<Hash class="mr-1 h-3 w-3 flex-shrink-0" />
			{:else}
				<Volume2 class="mr-1 h-3 w-3 flex-shrink-0" />
			{/if}
			<span class="truncate">
				{channel.name}
				{#if channel.unread}<span class="ml-1 text-red-500">*</span>{/if}
			</span>
			{#if channel.type === 'voice'}
				<span class="ml-auto text-xs text-gray-600">[{channel.users}]</span>
			{/if}
		</button>
	{/each}
</div>
<div class="border-t border-gray-400 p-2">
	<button class="w-full border border-gray-400 bg-gray-300 py-0.5 text-sm">Join Channel</button>
</div>
