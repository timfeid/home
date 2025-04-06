<script lang="ts">
	import { client } from '$lib/client';
	import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
	import { withPresence } from '$lib/presence.svelte';
	import type { Procedures } from '@feid/bindings';
	import { Users } from 'lucide-svelte';
	import { onMount } from 'svelte';

	let { niche }: { niche: Procedures['niche_find_by_slug']['output'] } = $props();
	let onlineUsers = $state<Procedures['channel_list_users']['output']['edges'][number]['node'][]>(
		[]
	);
	const presence = withPresence();

	onMount(async () => {
		const response = await client.channel_list_users.query({ niche_id: niche.id });
		if (response.status === 'ok') {
			onlineUsers = response.data.edges.map((edge) => edge.node);
		}
	});

	const getStatusColor = (activeClients: typeof presence.activeClients, userId: string) => {
		let status = 'offline';

		if (activeClients.some(({ user_id }) => user_id === userId)) {
			status = 'online';
		}

		switch (status) {
			case 'online':
				return 'bg-teal-500';
			case 'idle':
				return 'bg-yellow-500';
			case 'dnd':
				return 'bg-red-500';
			case 'offline':
				return 'bg-zinc-500';
		}
	};
</script>

<div class="h-full w-64 flex-shrink-0 overflow-y-auto bg-teal-900/5 px-5 py-3">
	<div class="mb-3 flex items-center">
		<Users class="mr-2 h-4 w-4 text-zinc-400" />
		<h3 class="font-mono text-sm font-medium">{onlineUsers.length}</h3>
	</div>

	<div class="mb-4 space-y-2">
		{#each onlineUsers as user}
			<div class="flex items-center">
				<div class="relative">
					<Avatar class="mr-2 h-7 w-7">
						<AvatarImage src={user.avatar_url} alt={user.username} />
						<AvatarFallback class="bg-teal-900 text-xs text-zinc-300">
							{user.username.substring(0, 1).toUpperCase()}
						</AvatarFallback>
					</Avatar>
					<span
						class={`absolute bottom-0 right-1 h-2 w-2 rounded-full ${getStatusColor(presence.activeClients, user.id)} ring-1 ring-zinc-800`}
					></span>
				</div>
				<span class="truncate font-mono text-sm">{user.username}</span>
				{#if user.role}
					<span
						class={`ml-1 rounded px-1 text-xs ${user.role === 'admin' ? 'bg-red-900 text-red-300' : 'bg-blue-900 text-blue-300'} font-mono`}
					>
						{user.role === 'admin' ? 'A' : 'M'}
					</span>
				{/if}
			</div>
		{/each}
	</div>

	<!-- <div class="mb-3 flex items-center">
			<Users class="mr-2 h-4 w-4 text-zinc-400" />
			<h3 class="font-mono text-sm font-medium">Offline — {offlineUsers.length}</h3>
		</div> -->
</div>
