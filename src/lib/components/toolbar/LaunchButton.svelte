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
	import DedicatedServerDialog from '$lib/components/dialogs/DedicatedServerDialog.svelte';
	import { pushInfoToast } from '$lib/toast';

	type Mode = 'vanilla' | 'modded' | 'server';

	const labels: Record<Mode, string> = {
		vanilla: m.toolBar_launch_vanilla(),
		modded: m.toolBar_launch_modded(),
		server: m.toolBar_launch_server()
	};

	const launchDropdownItems = $derived.by(() => {
		const items: ContextItem[] = [
			{
				label: labels.vanilla,
				onclick: () => {
					mode.current = 'vanilla';
					launchGame();
				}
			},
			{
				label: labels.modded,
				onclick: () => {
					mode.current = 'modded';
					launchGame();
				}
			}
		];

		if (games.active?.dedicatedServer) {
			items.push({
				label: labels.server,
				onclick: () => {
					mode.current = 'server';
					dedicatedServerDialogOpen = true;
				}
			});
		}

		return items;
	});

	let launchDialogOpen = $state(false);
	let launchDropdownOpen = $state(false);
	let launchOptionsDialogOpen = $state(false);
	let dedicatedServerDialogOpen = $state(false);
	let launchOptions = $state<LaunchOption[]>([]);

	const mode = new PersistedState<Mode>('launchMode', 'modded');

	const activeGameName = $derived(games.active?.name ?? m.unknown());

	async function launchGame() {
		if (mode.current === 'server') {
			await launchServer();
			return;
		}

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

	/// An already-configured server launches immediately; first-time setup
	/// opens the settings dialog instead.
	async function launchServer() {
		const settings = await api.profile.server.getSettings();

		if (settings === null || settings.serverName.trim() === '') {
			dedicatedServerDialogOpen = true;
			return;
		}

		try {
			await api.profile.server.launch(null, '', true);
			pushInfoToast({ message: m.toolBar_launchServer_started() });
		} catch {
			// invoke already reports the failure as an error toast.
		}
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

	$effect(() => {
		if (mode.current === 'server' && !games.active?.dedicatedServer) {
			mode.current = 'modded';
		}
	});
</script>

<div
	class="text-accent-500 *:hover:text-accent-400 group dark:*:group-hover:bg-primary-800 *:group-hover:bg-primary-200 flex shrink-0 gap-0.5 overflow-hidden rounded-lg font-semibold *:py-2"
>
	<button onclick={() => launchGame()} class="flex items-center pr-2 pl-4">
		<Icon icon="mdi:play-circle" class="mr-2 text-xl" />
		<span>
			{labels[mode.current]}
		</span>
	</button>

	<DropdownMenu.Root bind:open={launchDropdownOpen}>
		<DropdownMenu.Trigger class="pr-3 pl-2">
			<DropdownArrow open={launchDropdownOpen} />
		</DropdownMenu.Trigger>
		<DropdownMenu.Portal>
			<ContextMenuContent
				type="dropdown"
				items={launchDropdownItems}
				class="max-h-90 overflow-y-auto text-base"
			/>
		</DropdownMenu.Portal>
	</DropdownMenu.Root>
</div>

<Dialog
	title={(mode.current === 'vanilla'
		? m.toolBar_dialog_launch_vanilla_title
		: m.toolBar_dialog_launch_modded_title)({ name: activeGameName })}
	bind:open={launchDialogOpen}
>
	<p class="text-primary-500 dark:text-primary-400">
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

<DedicatedServerDialog bind:open={dedicatedServerDialogOpen} />
