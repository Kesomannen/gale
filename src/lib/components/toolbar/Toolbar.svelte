<script lang="ts">
	import Dialog from '$lib/components/ui/Dialog.svelte';

	import * as api from '$lib/api';
	import Icon from '@iconify/svelte';
	import GameSelect from '$lib/components/toolbar/GameSelect.svelte';
	import Updater from './Updater.svelte';
	import Syncer from './Syncer.svelte';
	import ProfilesDropdown from './ProfilesDropdown.svelte';
	import games from '$lib/state/game.svelte';
	import InstallPopover from './InstallPopover.svelte';
	import { message } from '@tauri-apps/plugin-dialog';
	import { m } from '$lib/paraglide/messages';
	import { gameIconSrc, timeSince } from '$lib/util';
	import { DropdownMenu } from 'bits-ui';
	import DropdownArrow from '../ui/DropdownArrow.svelte';
	import ContextMenuContent from '../ui/ContextMenuContent.svelte';
	import type { ContextItem } from '$lib/types';
	import { PersistedState } from 'runed';

	type Mode = 'vanilla' | 'modded';

	const labels: Record<Mode, string> = {
		vanilla: m.toolBar_launch_vanilla(),
		modded: m.toolBar_launch_modded()
	};

	const launchDropdownItems: ContextItem[] = [
		{
			label: labels['vanilla'],
			onclick: () => {
				mode.current = 'vanilla';
				launchGame();
			}
		},
		{
			label: labels['modded'],
			onclick: () => {
				mode.current = 'modded';
				launchGame();
			}
		}
	];

	let launchDialogOpen = $state(false);
	let launchDropdownOpen = $state(false);
	let gamesOpen = $state(false);

	const mode = new PersistedState<Mode>('launchMode', 'modded');

	let timeSinceGamesUpdate = $derived.by(() => {
		gamesOpen; // refresh whenever the dialog is opened
		return timeSince(games.lastUpdated);
	});

	const activeGameName = $derived(games.active?.name ?? m.unknown());

	async function launchGame() {
		if (await api.profile.install.hasPendingInstallations()) {
			await message(m.toolBar_launchGame_message());
			return;
		}

		launchDialogOpen = true;
		try {
			await api.profile.launch.launchGame(mode.current === 'vanilla');
		} catch {
			launchDialogOpen = false;
		}
	}
</script>

<div class="border-primary-600 bg-primary-900 flex h-12 shrink-0 flex-row border-t border-b">
	<div
		class="text-accent-400 *:hover:text-accent-400 border-primary-600 *:hover:bg-primary-800 flex shrink-0 items-stretch border-r font-semibold"
	>
		<button onclick={() => launchGame()} class="flex items-center pr-4 pl-6">
			<Icon icon="mdi:play-circle" class="mr-2 text-xl" />
			<span>
				{labels[mode.current]}
			</span>
		</button>

		<DropdownMenu.Root bind:open={launchDropdownOpen}>
			<DropdownMenu.Trigger class="px-3">
				<DropdownArrow open={launchDropdownOpen} />
			</DropdownMenu.Trigger>
			<ContextMenuContent
				type="dropdown"
				items={launchDropdownItems}
				class="max-h-90 overflow-y-auto text-base"
			/>
		</DropdownMenu.Root>
	</div>

	<button
		onclick={() => (gamesOpen = !gamesOpen)}
		class="group border-primary-600 text-primary-300 group-hover:text-primary-200 hover:bg-primary-800 flex shrink-0 items-center justify-between border-r pr-4 pl-2 font-semibold"
	>
		<img
			src={games.active ? gameIconSrc(games.active) : ''}
			class="mr-2 max-h-8 max-w-8 rounded-sm"
			alt={games.active?.name}
		/>

		{games.active?.name}

		<Icon
			icon="mdi:menu"
			class="text-primary-300 group-hover:text-primary-200 ml-6 shrink-0 text-lg"
		/>
	</button>

	<ProfilesDropdown />
	<Syncer />
	<InstallPopover />
	<Updater />
</div>

<Dialog
	title={(mode.current === 'vanilla'
		? m.toolBar_dialog_launch_vanilla_title
		: m.toolBar_dialog_launch_modded_title)({ name: activeGameName })}
	bind:open={launchDialogOpen}
>
	<p class="text-primary-400">
		{#if mode.current === 'modded'}
			{m.toolBar_dialog_launch_modded_content()}
		{/if}
	</p>
</Dialog>

<Dialog title={m.toolBar_dialog_games_title()} bind:open={gamesOpen}>
	<GameSelect onselect={() => (gamesOpen = false)} />
	<div class="text-primary-400 my-1 text-center text-sm">
		{m.toolBar_dialog_games_lastUpdated({ time: timeSinceGamesUpdate })}
	</div>
</Dialog>
