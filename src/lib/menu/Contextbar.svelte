<script lang="ts">
	import { confirm } from '@tauri-apps/plugin-dialog';

	import Popup from '$lib/components/Popup.svelte';
	import NewProfilePopup from '$lib/menu/NewProfilePopup.svelte';

	import {
		activeProfileIndex,
		activeGame,
		activeProfile,
		profiles,
		refreshProfiles,
		setActiveProfile
	} from '$lib/stores';
	import { invokeCommand } from '$lib/invoke';

	import Icon from '@iconify/svelte';
	import { Button, Dialog, DropdownMenu } from 'bits-ui';
	import GameSelection from '$lib/menu/GameSelection.svelte';
	import Updater from './Updater.svelte';
	import { dropTransition } from '$lib/transitions';
	import ProfileSelection from './ProfileSelection.svelte';

	let launchGamePopupOpen = false;
	let newProfilePopupOpen = false;

	let gamesOpen = false;
	let profilesOpen = false;

	function deleteProfile(index: number) {
		confirm(`Are you sure you want to delete ${profiles[index].name}?`).then(async (result) => {
			if (result) {
				await invokeCommand('delete_profile', { index });
				refreshProfiles();
			}
		});
	}

	function launchGame(vanilla: boolean) {
		invokeCommand('launch_game', { vanilla });
		launchGamePopupOpen = true;
	}
</script>

<div class="flex h-12 flex-shrink-0 flex-row border-b border-t border-slate-600 bg-slate-900">
	<div
		class="flex-shrink-0 border-r border-slate-600 pl-6 pr-8 text-accent-400 hover:bg-slate-800 hover:text-accent-400"
	>
		<Button.Root
			class="flex h-full cursor-default items-center font-semibold"
			on:click={() => launchGame(false)}
		>
			<Icon icon="mdi:play-circle" class="mr-2 text-xl" />
			Launch game
		</Button.Root>
	</div>

	<Button.Root
		on:click={() => (gamesOpen = !gamesOpen)}
		class="group flex flex-shrink-0 cursor-default items-center justify-between border-r border-slate-600 pl-2 pr-4 font-semibold text-slate-300 hover:bg-slate-800 group-hover:text-slate-200"
	>
		<img
			src="games/{$activeGame?.slug}.webp"
			class="mr-2 max-h-8 max-w-8 rounded"
			alt={$activeGame?.name}
		/>

		<div>{$activeGame?.name}</div>

		<Icon icon="mdi:menu" class="ml-6 flex-shrink-0 text-xl" />
	</Button.Root>

	<Button.Root
		on:click={() => (profilesOpen = !profilesOpen)}
		class="group flex flex-shrink-0 cursor-default items-center justify-between border-r border-slate-600 pl-6 pr-4 font-semibold text-slate-300 hover:bg-slate-800 group-hover:text-slate-200"
	>
		<div class="mr-auto flex-shrink truncate font-semibold">
			{$activeProfile?.name}
		</div>

		<div
			class="ml-6 mr-2 rounded bg-slate-800 px-2 py-0.5 text-sm font-medium group-hover:bg-slate-700"
		>
			{$activeProfile?.modCount}
		</div>

		<Icon icon="mdi:menu" class="flex-shrink-0 text-xl" />
	</Button.Root>

	<Updater />
</div>

<Popup title="Launching {$activeGame?.name}..." bind:open={launchGamePopupOpen}>
	<Dialog.Description class="text-slate-400">
		If the game is taking a while to start, it's probably because Steam is starting up.
	</Dialog.Description>
</Popup>

<Popup title="Select game to mod" bind:open={gamesOpen}>
	<GameSelection onSelect={() => (gamesOpen = false)} />
</Popup>

<Popup title="Select profile" bind:open={profilesOpen}>
	<ProfileSelection onSelect={() => (profilesOpen = false)} />
</Popup>

<NewProfilePopup bind:open={newProfilePopupOpen} />
