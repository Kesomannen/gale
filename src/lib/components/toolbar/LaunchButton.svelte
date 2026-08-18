<script lang="ts">
	import Dialog from '$lib/components/ui/Dialog.svelte';
	import LaunchOptionsDialog from '$lib/components/dialogs/LaunchOptionsDialog.svelte';

	import * as api from '$lib/api';
	import Icon from '@iconify/svelte';
	import games from '$lib/state/game.svelte';
	import { message } from '@tauri-apps/plugin-dialog';
	import { m } from '$lib/paraglide/messages';
	import type { LaunchOption } from '$lib/types';
	import { DropdownMenu } from 'bits-ui';
	import DropdownArrow from '../ui/DropdownArrow.svelte';
	import ContextMenuContent from '../ui/ContextMenuContent.svelte';
	import { type ContextItem } from '$lib/types';
	import { PersistedState } from '$lib/state/persisted-state.svelte';

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
	let launchOptionsDialogOpen = $state(false);
	let launchOptions = $state<LaunchOption[]>([]);

	const mode = new PersistedState<Mode>('launchMode', 'modded');

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

<div
	class="text-accent-500 *:hover:text-accent-400 *:group-hover:bg-primary-800 group flex shrink-0 gap-0.5 overflow-hidden rounded-lg font-semibold *:py-2"
>
	<button onclick={() => launchGame()} class="flex items-center pr-2 pl-4">
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

<LaunchOptionsDialog
	bind:open={launchOptionsDialogOpen}
	options={launchOptions}
	gameName={games.active?.name ?? ''}
	onselect={handleLaunchOptionSelect}
/>
