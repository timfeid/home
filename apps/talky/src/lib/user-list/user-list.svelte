<script lang="ts">
	// Mock user data with different statuses
	const users = [
		{ id: 1, name: '@Admin', status: 'op', speaking: false },
		{ id: 2, name: '+GamerGirl', status: 'voice', speaking: true },
		{ id: 3, name: 'CoolDude', status: 'normal', speaking: false },
		{ id: 4, name: 'NightOwl', status: 'away', speaking: false },
		{ id: 5, name: 'SilentNinja', status: 'normal', speaking: false }
	];

	function getStatusColor(status: string) {
		switch (status) {
			case 'op':
				return 'text-red-600';
			case 'voice':
				return 'text-blue-600';
			case 'away':
				return 'text-gray-500';
			default:
				return 'text-black';
		}
	}
</script>

<div class="bg-blue-800 p-1 text-xs font-bold text-white">USERS (5)</div>
<div class="h-full overflow-y-auto">
	{#each users as user (user.id)}
		<div
			class="px-2 py-0.5 text-sm {user.speaking
				? 'bg-yellow-200'
				: 'hover:bg-gray-300'} {getStatusColor(user.status)}"
		>
			{user.name}
			{#if user.speaking}<span class="ml-1 text-xs">[talking]</span>{/if}
			{#if user.status === 'away'}<span class="ml-1 text-xs">[away]</span>{/if}
		</div>
	{/each}
</div>
