<script lang="ts">
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import LaunchOptionsDialog from '$lib/components/dialogs/LaunchOptionsDialog.svelte';

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
	import type { LaunchOption } from '$lib/types';
	import { DropdownMenu } from 'bits-ui';
	import DropdownArrow from '../ui/DropdownArrow.svelte';
	import ContextMenuContent from '../ui/ContextMenuContent.svelte';
	import { type ContextItem } from '$lib/types';
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
	let launchOptionsDialogOpen = $state(false);
	let launchOptions = $state<LaunchOption[]>([]);

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

		const prefs = await api.prefs.get();
		prefs.gamePrefs = new Map(Object.entries(prefs.gamePrefs));
		const currentGameSlug = games.active?.slug;
		if (!currentGameSlug) return;

		const gamePrefs = prefs.gamePrefs.get(currentGameSlug);

		if (
			gamePrefs &&
			gamePrefs.launchMode.type === 'launcher' &&
			gamePrefs.platform === 'steam' &&
			gamePrefs.showSteamLaunchOptions
		) {
			const options = await api.profile.launch.getSteamLaunchOptions();

			if (options.length > 0) {
				launchOptions = options;
				launchOptionsDialogOpen = true;
				return;
			}
		}

		await doLaunch();
	}

	async function doLaunch(args?: string) {
		launchDialogOpen = true;
		try {
			await api.profile.launch.launchGame(mode.current === 'vanilla', args);
		} catch {
			launchDialogOpen = false;
		}
	}

	function handleLaunchOptionSelect(args: string) {
		doLaunch(args);
	}
</script>

<div class="flex shrink-0 items-center gap-2 px-3 py-1">
	<div
		class="text-accent-500 *:hover:text-accent-400 *:group-hover:bg-primary-800 group flex shrink-0 gap-0.5 overflow-hidden rounded-lg font-semibold *:py-2"
	>
		<button onclick={() => launchGame()} class="flex items-center pr-3 pl-6">
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
		class="group text-primary-300 group-hover:text-primary-200 hover:bg-primary-800 flex shrink-0 items-center rounded-lg px-4 py-2 font-semibold"
	>
		<img
			src={games.active ? gameIconSrc(games.active) : ''}
			class="mr-2 size-8 rounded"
			alt={games.active?.name}
		/>

		<div class="mr-4 hidden lg:block">{games.active?.name}</div>

		<Icon icon="mdi:menu" class="text-primary-300 group-hover:text-primary-200  shrink-0 text-lg" />
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

<LaunchOptionsDialog
	bind:open={launchOptionsDialogOpen}
	options={launchOptions}
	gameName={games.active?.name ?? ''}
	onselect={handleLaunchOptionSelect}
/>
