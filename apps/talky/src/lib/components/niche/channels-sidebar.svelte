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
		Hash,
		MessageSquare,
		Plus,
		RadioTower,
		Settings
	} from 'lucide-svelte';
	import Badge from '../ui/badge/badge.svelte';
	import type { Niche } from './niche.svelte';
	import type { OutgoingMessage } from '@talky/soundhouse';
	import * as Collapsible from '$lib/components/ui/collapsible/index.js';
	import SettingsDialog from '../settings/settings-dialog.svelte';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import GroupedAvatars from '../ui/avatar/grouped-avatars.svelte';

	let { niche }: { niche: Niche } = $props();
	let categories = $state<Procedures['category_list']['output']['edges'][number]['node'][]>([]);
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

	let creatingLobbyName = $state('');
	async function createLobby(channelId: string) {
		const randomChannelName = `${user.user!.sub}-${Math.random().toString(36).substring(2, 7)}`;
		creatingLobbyName = randomChannelName;
		try {
			const response = wrapResponse(
				await client.lobby_create_temporary.query({
					name: randomChannelName,
					channel_id: channelId
				})
			);

			categories.push(response);
		} catch (e) {}

		creatingLobbyName = '';
	}

	async function resetChannels() {
		const response = wrapResponse(await client.category_list.query({ niche_id: niche.id }));
		pageInfo = response.page_info;
		categories = response.edges.map((edge) => edge.node);
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
	<Sidebar.Content>
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
												props.class || '',
												page.params.channel === channel.slug ? 'bg-sidebar-accent/55' : ''
											)}
										>
											<div class="flex flex-1 items-center overflow-hidden">
												{#if channel.type === 'chat'}
													<Hash class="mr-1.5 h-4 w-4 flex-shrink-0" />
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
								<Sidebar.MenuSub>
									<Sidebar.MenuItem>
										<Sidebar.MenuButton>
											<div class="flex flex-1 items-center overflow-hidden text-purple-200">
												<AudioLines class="mr-1.5 h-4 w-4 flex-shrink-0" />
												<span class="truncate font-light">dazed-k2zs5</span>
											</div>
											{#snippet tooltipContent()}
												dazed
											{/snippet}
											<GroupedAvatars
												avatars={[
													{ user_id: 'dazed', type: 'UserResource' },
													{ user_id: 'jumbo', type: 'UserResource' },
													{ user_id: 'erickuh', type: 'UserResource' },
													{ user_id: 'africkuh', type: 'UserResource' }
												]}
											/>
										</Sidebar.MenuButton>
									</Sidebar.MenuItem>
								</Sidebar.MenuSub>
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
