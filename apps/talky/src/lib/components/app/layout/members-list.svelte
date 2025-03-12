<script lang="ts">
	// Import Svelte components (ensure these components exist in your project)
	import { X } from 'lucide-svelte';
	import { ScrollArea } from '../../ui/scroll-area';
	import { Avatar, AvatarFallback, AvatarImage } from '../../ui/avatar';
	import type { LobbyData } from '../lobby/lobby';

	let { lobby, onClose }: { lobby: LobbyData; onClose: () => void } = $props();

	let roles = $derived(lobby.member_list);
</script>

<div class="flex w-60 flex-col bg-black">
	<!-- Header (hidden on larger screens) -->
	<div
		class="flex h-12 items-center justify-between border-b border-[#202225] px-4 shadow-sm md:hidden"
	>
		<h2 class="font-semibold text-white">Members</h2>
		<button on:click={onClose} class="text-gray-400 hover:text-white">
			<X class="h-5 w-5" />
		</button>
	</div>

	<ScrollArea class="flex-1">
		<div class="p-4">
			{#each roles as role, roleIndex}
				<div class="mb-6">
					<h3 class="mb-2 text-xs font-semibold uppercase">
						{role.name} — {role.members.length}
					</h3>

					<div class="space-y-2">
						{#each role.members as member, memberIndex}
							<div
								class="group flex cursor-pointer items-center rounded px-2 py-1 hover:bg-neutral-900"
							>
								<div class="relative mr-3">
									<Avatar class="h-8 w-8">
										<AvatarImage src={member.avatar} />
										<AvatarFallback>{member.name.charAt(0)}</AvatarFallback>
									</Avatar>
									<!-- {#if !role.is_offline}
										<div
											class="absolute bottom-0 right-0 h-3 w-3 rounded-full bg-green-500 ring-2 ring-[#2f3136] group-hover:ring-[#36393f]"
										></div>
									{/if} -->
								</div>

								<div>
									<p class="text-sm font-medium {member.color || 'text-white'}">
										{member.name}
									</p>

									<!-- {#if member.presence_text}
										<p class="text-xs text-gray-400">
											{member.presence_text}
										</p>
									{/if} -->
								</div>
							</div>
						{/each}
					</div>
				</div>
			{/each}
		</div>
	</ScrollArea>
</div>
