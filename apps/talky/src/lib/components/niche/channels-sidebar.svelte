<script lang="ts">
	import { page } from '$app/state';
	import { client, wrapResponse } from '$lib/client';
	import { Avatar, AvatarFallback } from '$lib/components/ui/avatar';
	import { Button } from '$lib/components/ui/button';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import { withPresence } from '$lib/presence.svelte';
	import { user } from '$lib/user.svelte';
	import { cn } from '$lib/utils';
	import type { PageInfo, Procedures } from '@feid/bindings';
	import { AudioLines, ChevronDown, ChevronUp, Hash, MessageSquare, Plus } from 'lucide-svelte';
	import Badge from '../ui/badge/badge.svelte';
	import type { Niche } from './niche.svelte';

	let { niche }: { niche: Niche } = $props();
	let channels = $state<Procedures['channel_list']['output']['edges'][number]['node'][]>([]);
	let channelsSlug = $state('');

	let pageInfo: PageInfo | undefined = $state();
	const presence = withPresence();

	async function joinChannel(channelId: string) {
		if (!user.user) {
			return;
		}

		presence.joinChannel(channelId);
	}

	$effect(() => {
		if (presence.isConnected && presence.currentNicheId !== niche.id) {
			presence.sendMessage({ type: 'update_niche', niche_id: niche.id });
			presence.currentNicheId = niche.id;
		}
	});

	$effect(() => {
		let channel =
			presence.channelConnection.id && presence.activeChannels[presence.channelConnection.id];
		if (
			channel &&
			presence.channelConnection.status === 'init' &&
			channel.users.some((cu) => cu.user.user_id === user.user!.sub)
		) {
			presence.channelConnection.status = 'connected';
			presence.connected();
		}
	});

	let creatingTemporaryChannelName = $state('');
	async function createTemporaryChannel() {
		const randomChannelName = `${user.user!.sub}-${Math.random().toString(36).substring(2, 7)}`;
		creatingTemporaryChannelName = randomChannelName;
		try {
			const response = wrapResponse(
				await client.channel_create_temporary.query({
					name: randomChannelName,
					niche_id: niche.id,
					type: 'multi_media'
				})
			);

			channels.push(response);
		} catch (e) {}

		creatingTemporaryChannelName = '';
	}

	async function resetChannels() {
		const response = wrapResponse(await client.channel_list.query({ niche_id: niche.id }));
		pageInfo = response.page_info;
		channels = response.edges.map((edge) => edge.node);
	}

	const permChannels = $derived.by(() => {
		return channels.filter((channel) => !channel.is_temporary);
	});
	const temporaryChannels = $derived.by(() => {
		return channels.filter((channel) => channel.is_temporary);
	});

	$effect(() => {
		if (channelsSlug !== niche.slug) {
			resetChannels();
		}
	});
</script>

<Sidebar.Root>
	<Sidebar.Header>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<DropdownMenu.Root>
					<DropdownMenu.Trigger>
						{#snippet child({ props })}
							<Sidebar.MenuButton {...props}>
								{niche.name}
								<ChevronDown class="ml-auto" />
							</Sidebar.MenuButton>
						{/snippet}
					</DropdownMenu.Trigger>
					<DropdownMenu.Content class="w-[--bits-dropdown-menu-anchor-width]">
						<DropdownMenu.Item>
							{#snippet child({ props })}
								<a href="/of/devils/gameday" {...props}>Devils</a>
							{/snippet}
						</DropdownMenu.Item>
						<DropdownMenu.Item>
							{#snippet child({ props })}
								<a href="/of/pirate/movie-releases" {...props}>Pirate</a>
							{/snippet}
						</DropdownMenu.Item>
					</DropdownMenu.Content>
				</DropdownMenu.Root>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Header>
	<Sidebar.Content>
		<Sidebar.Group>
			<Sidebar.GroupLabel>Temporary Channels</Sidebar.GroupLabel>
			<Sidebar.GroupAction onclick={createTemporaryChannel} title="Create temporary channel">
				<Plus class="!size-3.5" />
				<span class="sr-only">Create temporary channel</span>
			</Sidebar.GroupAction>
			<Sidebar.GroupContent>
				<Sidebar.Menu>
					{#if creatingTemporaryChannelName}
						<Sidebar.MenuItem>
							<Sidebar.MenuButton class={'opacity-50'}>
								<div class="flex flex-1 items-center overflow-hidden">
									<AudioLines class="mr-1.5 h-4 w-4 flex-shrink-0" />
									<span class="truncate font-light">{creatingTemporaryChannelName}</span>
								</div>
							</Sidebar.MenuButton>
						</Sidebar.MenuItem>
					{/if}
					{#each temporaryChannels as channel}
						<Sidebar.MenuItem>
							<Sidebar.MenuButton onclick={() => joinChannel(channel.id)}>
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
							</Sidebar.MenuButton>
							{@const room = presence.activeChannels[channel.id]}
							{#if room}
								<Sidebar.MenuSub>
									{#each room.users as u}
										<Sidebar.MenuSubItem>
											<Sidebar.MenuSubButton class="h-auto py-1">
												<Avatar class="mr-2 h-6 w-6">
													<!-- <AvatarImage src={user.user!.avatar_url} alt={user.user!.username} /> -->
													<AvatarFallback class="bg-teal-900 text-xs text-zinc-300">
														{u.user.user_id.substring(0, 1).toUpperCase()}
													</AvatarFallback>
												</Avatar>
												<div class="mr-auto">
													{u.user.user_id}
												</div>
											</Sidebar.MenuSubButton>
										</Sidebar.MenuSubItem>
									{/each}
								</Sidebar.MenuSub>
							{/if}
							{#if presence.channelConnection.id === channel.id && presence.channelConnection.status !== 'connected'}
								<Sidebar.MenuSub>
									<Sidebar.MenuSubItem>
										<Sidebar.MenuSubButton class="h-auto py-1 opacity-20">
											<Avatar class="mr-2 h-6 w-6">
												<!-- <AvatarImage src={user.user!.avatar_url} alt={user.user!.username} /> -->
												<AvatarFallback class="bg-teal-900 text-xs text-zinc-300">
													{user.user!.sub.substring(0, 1).toUpperCase()}
												</AvatarFallback>
											</Avatar>
											<div class="mr-auto">
												{user.user!.sub}
											</div>
											<Badge variant="outline" class="border-yellow-500 px-1.5 py-0 uppercase">
												{presence.channelConnection.status}
											</Badge>
										</Sidebar.MenuSubButton>
									</Sidebar.MenuSubItem>
								</Sidebar.MenuSub>
							{/if}
						</Sidebar.MenuItem>
					{/each}
				</Sidebar.Menu>
			</Sidebar.GroupContent>
		</Sidebar.Group>
		<Sidebar.Group>
			<Sidebar.GroupLabel>Permanent Channels</Sidebar.GroupLabel>
			<Sidebar.GroupContent>
				<Sidebar.Menu>
					{#each permChannels as channel}
						<Sidebar.MenuItem>
							<Sidebar.MenuButton>
								{#snippet child({ props })}
									<a
										href="/of/{niche.slug}/{channel.slug}"
										{...props}
										class={cn(
											props.class || '',
											page.params.channel === channel.slug ? 'bg-sidebar-accent/55' : ''
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
									</a>
								{/snippet}
							</Sidebar.MenuButton>
						</Sidebar.MenuItem>
					{/each}
				</Sidebar.Menu>
			</Sidebar.GroupContent>
		</Sidebar.Group>
	</Sidebar.Content>
	<Sidebar.Footer>
		{#if user.user}
			<Sidebar.Menu>
				<Sidebar.MenuItem>
					<DropdownMenu.Root>
						<DropdownMenu.Trigger>
							{#snippet child({ props })}
								<Sidebar.MenuButton
									{...props}
									class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground"
								>
									{user.user!.sub}
									<ChevronUp class="ml-auto" />
								</Sidebar.MenuButton>
							{/snippet}
						</DropdownMenu.Trigger>
						<DropdownMenu.Content side="top" class="w-[--bits-dropdown-menu-anchor-width]">
							<DropdownMenu.Item>
								<span>Sign out</span>
							</DropdownMenu.Item>
						</DropdownMenu.Content>
					</DropdownMenu.Root>
				</Sidebar.MenuItem>
			</Sidebar.Menu>
		{:else}
			<div class="flex gap-1.5">
				<Button class="w-1/2" variant="secondary" href="/login">Log in</Button>
				<Button class="w-1/2" variant="link" href="/register">Sign up</Button>
			</div>
		{/if}
	</Sidebar.Footer></Sidebar.Root
>
