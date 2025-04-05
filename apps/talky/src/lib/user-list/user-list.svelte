<script lang="ts">
	import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
	import { Separator } from '$lib/components/ui/separator';
	import { Users } from 'lucide-svelte';

	type User = {
		id: string;
		name: string;
		avatar: string;
		status: 'online' | 'idle' | 'dnd' | 'offline';
		role?: 'admin' | 'mod';
	};

	const users: User[] = [
		{
			id: '1',
			name: 'DevilsFan83',
			avatar: '/placeholder.svg?height=40&width=40',
			status: 'online',
			role: 'admin'
		},
		{
			id: '2',
			name: 'JerseyDevil',
			avatar: '/placeholder.svg?height=40&width=40',
			status: 'online'
		},
		{
			id: '3',
			name: 'HockeyAnalyst',
			avatar: '/placeholder.svg?height=40&width=40',
			status: 'online',
			role: 'mod'
		},
		{ id: '4', name: 'DevilsDaily', avatar: '/placeholder.svg?height=40&width=40', status: 'idle' },
		{ id: '5', name: 'PuckReport', avatar: '/placeholder.svg?height=40&width=40', status: 'dnd' },
		{
			id: '6',
			name: 'GoalieGuru',
			avatar: '/placeholder.svg?height=40&width=40',
			status: 'offline'
		},
		{
			id: '7',
			name: 'DefenseFirst',
			avatar: '/placeholder.svg?height=40&width=40',
			status: 'offline'
		}
	];

	const getStatusColor = (status: User['status']) => {
		switch (status) {
			case 'online':
				return 'bg-zinc-500';
			case 'idle':
				return 'bg-yellow-500';
			case 'dnd':
				return 'bg-red-500';
			case 'offline':
				return 'bg-zinc-500';
		}
	};

	const onlineUsers = users.filter((user) => user.status !== 'offline');
	const offlineUsers = users.filter((user) => user.status === 'offline');
</script>

<div class="h-full w-64 flex-shrink-0 overflow-y-auto bg-teal-900/5 px-5 py-3">
	<div class="mb-3 flex items-center">
		<Users class="mr-2 h-4 w-4 text-zinc-400" />
		<h3 class="font-mono text-sm font-medium">Online — {onlineUsers.length}</h3>
	</div>

	<div class="mb-4 space-y-2">
		{#each onlineUsers as user}
			<div class="flex items-center">
				<div class="relative">
					<Avatar class="mr-2 h-7 w-7">
						<AvatarImage src={user.avatar} alt={user.name} />
						<AvatarFallback class="bg-teal-900 text-xs text-zinc-300">
							{user.name.substring(0, 2).toUpperCase()}
						</AvatarFallback>
					</Avatar>
					<span
						class={`absolute bottom-0 right-1 h-2 w-2 rounded-full ${getStatusColor(user.status)} ring-1 ring-zinc-800`}
					></span>
				</div>
				<span class="truncate font-mono text-sm">{user.name}</span>
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

	{#if offlineUsers.length > 0}
		<div class="mb-3 flex items-center">
			<Users class="mr-2 h-4 w-4 text-zinc-400" />
			<h3 class="font-mono text-sm font-medium">Offline — {offlineUsers.length}</h3>
		</div>

		<div class="space-y-2">
			{#each offlineUsers as user}
				<div class="flex items-center opacity-60">
					<div class="relative">
						<Avatar class="mr-2 h-7 w-7">
							<AvatarImage src={user.avatar} alt={user.name} />
							<AvatarFallback class="bg-teal-900 text-xs text-zinc-300">
								{user.name.substring(0, 2).toUpperCase()}
							</AvatarFallback>
						</Avatar>
						<span
							class={`absolute bottom-0 right-1 h-2 w-2 rounded-full ${getStatusColor(user.status)} ring-1 ring-zinc-800`}
						></span>
					</div>
					<span class="truncate font-mono text-sm">{user.name}</span>
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
	{/if}
</div>
