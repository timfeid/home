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
	import {
		AudioLines,
		ChevronDown,
		ChevronUp,
		Cog,
		DiamondPlus,
		Hash,
		MessageSquare,
		PhoneCall,
		Plus,
		RadioTower,
		Settings,
		UserRoundPlus
	} from 'lucide-svelte';
	import Badge from '../ui/badge/badge.svelte';
	import type { Niche } from './niche.svelte';
	import type { OutgoingMessage, RoomResource } from '@talky/soundhouse';
	import * as Collapsible from '$lib/components/ui/collapsible/index.js';
	import SettingsDialog from '../settings/settings-dialog.svelte';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import GroupedAvatars from '../ui/avatar/grouped-avatars.svelte';
	import * as Tooltip from '../ui/tooltip';
	import { mergeProps } from 'bits-ui';

	let { niche }: { niche: Niche } = $props();
	let categories = $state<Procedures['category_list']['output']['edges'][number]['node'][]>([]);
	let channelsSlug = $state('');

	let pageInfo: PageInfo | undefined = $state();
	const presence = withPresence();

	async function joinLobby(lobbyId: string) {
		if (!user.user) {
			return;
		}

		presence.joinLobby(lobbyId);
	}

	$effect(() => {
		if (presence.isConnected && presence.currentNicheId !== niche.id) {
			presence.sendMessage({ type: 'update_niche', niche_id: niche.id } as OutgoingMessage);
			presence.currentNicheId = niche.id;
		}
	});

	$effect(() => {
		let channel =
			presence.channelConnection.id && presence.activeChannels[presence.channelConnection.id];
		if (channel && presence.channelConnection.status === 'init' && channel.users[user.user!.sub]) {
			presence.channelConnection.status = 'connected';
			presence.connected();
		}
	});

	let creatingLobby = $state({ name: '', channelId: '' });
	async function createLobby(channel: unknown) {
		console.log(channel);
		const randomChannelName = `${user.user!.sub}-${Math.random().toString(36).substring(2, 7)}`;
		creatingLobby.name = randomChannelName;
		creatingLobby.channelId = channel.id;
		try {
			const response = wrapResponse(
				await client.lobby_create_temporary.query({
					name: randomChannelName,
					channel_id: channel.id
				})
			);
			channel.lobbies.push(response);
			await joinLobby(response.id);
		} catch (e) {}

		creatingLobby.channelId = '';
	}

	async function resetChannels() {
		const response = wrapResponse(await client.category_list.query({ niche_id: niche.id }));
		pageInfo = response.page_info;
		categories = response.edges.map((edge) => edge.node);
	}

	function getUsers(lobby: RoomResource | undefined) {
		if (!lobby || !Object.keys(lobby.users).length) {
			return [];
		}

		return Object.values(lobby.users)
			.map((ru) => ru?.[0].user)
			.filter((u) => !!u);
	}

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
								<a href="/of/hockey/devils" {...props}>Hockey</a>
							{/snippet}
						</DropdownMenu.Item>
						<DropdownMenu.Item>
							{#snippet child({ props })}
								<a href="/of/pirate/movies" {...props}>Pirate</a>
							{/snippet}
						</DropdownMenu.Item>
					</DropdownMenu.Content>
				</DropdownMenu.Root>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Header>
	<Sidebar.Content class="font-mono">
		{#each categories as category}
			<Sidebar.Group>
				<Sidebar.GroupLabel>{category.name}</Sidebar.GroupLabel>
				<Sidebar.GroupContent>
					<Sidebar.Menu>
						{#each category.channels as channel}
							<Sidebar.MenuItem>
								<Sidebar.MenuButton>
									{#snippet child({ props })}
										<a
											href="/of/{niche.slug}/{channel.slug}"
											{...props}
											class={cn(
												'flex flex-1 items-center overflow-hidden',
												props.class || '',
												page.params.channel === channel.slug
													? 'bg-sidebar-accent hover:bg-sidebar-accent'
													: ''
											)}
										>
											{#if channel.type === 'chat'}
												<Hash class="mr-1.5 h-4 w-4 flex-shrink-0" />
											{:else if channel.type === 'feed'}
												<Hash class="mr-1.5 h-4 w-4 flex-shrink-0" />
											{:else}
												<AudioLines class="mr-1.5 h-4 w-4 flex-shrink-0" />
											{/if}
											<span class="truncate">{channel.name}</span>
										</a>
									{/snippet}
								</Sidebar.MenuButton>

								<Sidebar.MenuAction showOnHover class="-mt-0.5 !h-6 !w-6">
									{#snippet child({ props: actionProps })}
										<Tooltip.Root delayDuration={1000}>
											<Tooltip.Trigger>
												{#snippet child({ props: tooltipProps })}
													<button
														{...mergeProps(tooltipProps, actionProps)}
														onclick={() => createLobby(channel)}
													>
														<DiamondPlus class="!size-4" />
														<span class="sr-only"></span>
													</button>
												{/snippet}
											</Tooltip.Trigger>
											<Tooltip.Content>Create a lobby</Tooltip.Content>
										</Tooltip.Root>
									{/snippet}
								</Sidebar.MenuAction>

								{#if channel.lobbies.length > 0 || creatingLobby.channelId === channel.id}
									<Sidebar.MenuSub>
										<Sidebar.MenuItem>
											{#if creatingLobby.channelId === channel.id}
												<Sidebar.MenuButton>
													<div class="flex flex-1 items-center overflow-hidden text-purple-100">
														<AudioLines class="mr-1.5 h-4 w-4 flex-shrink-0" />
														<span class="truncate font-light">{creatingLobby.name}</span>
													</div>
													{#snippet tooltipContent()}
														{creatingLobby.name}
													{/snippet}
													<GroupedAvatars max={3} avatars={[{ user_id: user.user!.sub }]} />
												</Sidebar.MenuButton>
											{/if}
										</Sidebar.MenuItem>
										{#each channel.lobbies as lobby}
											<Sidebar.MenuItem>
												<Sidebar.MenuButton>
													<div class="flex flex-1 items-center overflow-hidden text-purple-100">
														<AudioLines class="mr-1.5 h-4 w-4 flex-shrink-0" />
														<span class="truncate font-light">{lobby.name}</span>
													</div>
													{#snippet tooltipContent()}
														{lobby.name}
													{/snippet}
													<GroupedAvatars
														max={3}
														avatars={getUsers(withPresence().activeChannels[lobby.id])}
													/>
												</Sidebar.MenuButton>

												<Sidebar.MenuAction showOnHover class="-mt-0.5 !h-6 !w-6">
													{#snippet child({ props: actionProps })}
														<Tooltip.Root delayDuration={1000}>
															<Tooltip.Trigger>
																{#snippet child({ props: tooltipProps })}
																	<button
																		{...mergeProps(tooltipProps, actionProps)}
																		onclick={() => joinLobby(lobby.id)}
																	>
																		<PhoneCall class="!size-4" />
																		<span class="sr-only">Toggle</span>
																	</button>
																{/snippet}
															</Tooltip.Trigger>
															<Tooltip.Content>Join this lobby</Tooltip.Content>
														</Tooltip.Root>
													{/snippet}
												</Sidebar.MenuAction>
											</Sidebar.MenuItem>
										{/each}
									</Sidebar.MenuSub>
								{/if}

								<!-- <Sidebar.MenuSub>
									{#each [[{ user: { user_id: 'dazed' } }]] as u}
										{#if u}
											<Sidebar.MenuSubItem>
												<Sidebar.MenuSubButton class="h-auto py-1">
													<Avatar class="mr-2 h-6 w-6">
														<AvatarFallback class="bg-teal-900 text-xs text-zinc-300">
															{u[0].user.user_id.substring(0, 1).toUpperCase()}
														</AvatarFallback>
													</Avatar>
													<div class="mr-auto">
														{u[0].user.user_id}
													</div>
												</Sidebar.MenuSubButton>
											</Sidebar.MenuSubItem>
										{/if}
									{/each}
								</Sidebar.MenuSub> -->
							</Sidebar.MenuItem>
						{/each}
					</Sidebar.Menu>
				</Sidebar.GroupContent>
			</Sidebar.Group>
		{/each}
	</Sidebar.Content>
	<Sidebar.Footer>
		{#if user.user}
			<div class="flex">
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
				<div class="no-shrink ml-0.5 w-8">
					<SettingsDialog>
						{#snippet child({ props })}
							<Button {...props} variant="ghost" class="h-8 w-8" size="icon">
								<Settings class="size-5" />
							</Button>
						{/snippet}
					</SettingsDialog>
				</div>
			</div>
		{:else}
			<div class="flex gap-1.5">
				<Button class="w-1/2" variant="secondary" href="/login">Log in</Button>
				<Button class="w-1/2" variant="link" href="/register">Sign up</Button>
			</div>
		{/if}
	</Sidebar.Footer>
</Sidebar.Root>
