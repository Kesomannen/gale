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
	import { gameIconSrc, timeSince } from '$lib/util';

	let launchDialogOpen = $state(false);
	let gamesOpen = $state(false);

	let timeSinceGamesUpdate = $derived.by(() => {
		gamesOpen; // refresh whenever the dialog is opened
		return timeSince(games.lastUpdated);
	});

	async function launchGame() {
		if (await api.profile.install.hasPendingInstallations()) {
			await message('Please wait for mod installations to complete before launching.');
			return;
		}

		launchDialogOpen = true;
		try {
			await api.profile.launch.launchGame();
		} catch {
			launchDialogOpen = false;
		}
	}
</script>

<div class="border-primary-600 bg-primary-900 flex h-12 shrink-0 flex-row border-t border-b">
	<button
		class="text-accent-400 hover:text-accent-400 border-primary-600 hover:bg-primary-800 flex shrink-0 items-center border-r pr-8 pl-6 font-semibold"
		onclick={launchGame}
	>
		<Icon icon="mdi:play-circle" class="mr-2 text-xl" />
		Launch game
	</button>

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

<Dialog title="Launching {games.active?.name}..." bind:open={launchDialogOpen}>
	<p class="text-primary-400">
		This might take a few minutes depending on the size of your profile.
	</p>
</Dialog>

<Dialog title="Select game to mod" bind:open={gamesOpen}>
	<GameSelect onselect={() => (gamesOpen = false)} />
	<div class="text-primary-400 my-1 text-center text-sm">
		Last updated {timeSinceGamesUpdate} ago
	</div>
</Dialog>
