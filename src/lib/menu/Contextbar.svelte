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
	import { fade, fly } from 'svelte/transition';
	import GameSelection from '$lib/components/GameSelection.svelte';
	import Updater from './Updater.svelte';
	import { quadOut } from 'svelte/easing';
	import { dropTransition } from '$lib/transitions';

	let launchGamePopupOpen = false;
	let newProfilePopupOpen = false;

	let gamesOpen = false;
	let profilesOpen = false;

	function deleteProfile(index: number) {
		confirm(`Are you sure you want to delete ${profiles[index].name}?`, {
			title: 'Delete profile'
		}).then(async (result) => {
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

<div class="flex h-12 flex-shrink-0 flex-row border-b border-t border-gray-600 bg-gray-900">
	<div
		class="text-accent-400 hover:text-accent-400 flex-shrink-0 border-r border-gray-600 pl-6 pr-8 hover:bg-gray-800"
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
		class="group flex flex-shrink-0 cursor-default items-center justify-between border-r border-gray-600 pl-2 pr-4 font-semibold text-gray-300 hover:bg-gray-800 group-hover:text-gray-200"
	>
		{#if $activeGame}
			<img
				src="games/{$activeGame.id}.webp"
				class="mr-2 max-h-8 max-w-8 rounded"
				alt={$activeGame.displayName}
			/>

			{$activeGame.displayName}
		{:else}
			Loading...
		{/if}

		<Icon
			icon="mdi:menu"
			class="ml-6 flex-shrink-0 text-xl text-gray-300 transition-all group-hover:text-gray-200"
		/>
	</Button.Root>

	<DropdownMenu.Root bind:open={profilesOpen}>
		<DropdownMenu.Trigger
			class="group flex min-w-40 flex-shrink cursor-default items-center border-r border-gray-600 pl-6 
						pr-4 text-gray-300 hover:bg-gray-800 group-hover:text-gray-200"
		>
			<span class="mr-auto flex-shrink truncate font-semibold">
				{$activeProfile?.name}
			</span>

			<div
				class="ml-6 mr-2 rounded bg-gray-800 px-2 py-0.5 text-sm font-medium group-hover:bg-gray-700"
			>
				{$activeProfile?.modCount}
			</div>

			<Icon
				icon="mdi:expand-more"
				class="flex-shrink-0 origin-center transform text-xl transition-all {profilesOpen
					? 'rotate-180'
					: 'rotate-0'}"
			/>
		</DropdownMenu.Trigger>
		<DropdownMenu.Content
			class="flex max-h-[80lvh] min-w-40 flex-col gap-0.5 overflow-y-auto rounded-b-lg border border-gray-600 bg-gray-800 p-1 shadow-xl"
			{...dropTransition}
		>
			{#each profiles as profile, i}
				<DropdownMenu.Item
					class="group flex cursor-default items-center rounded-md py-1 pl-3 pr-1 text-left hover:bg-gray-700
						{i == activeProfileIndex
						? 'font-medium text-gray-300 hover:text-gray-200'
						: 'text-gray-400 hover:text-gray-300'}"
					on:click={() => {
						setActiveProfile(i);
						profilesOpen = false;
					}}
				>
					<span class="mr-3 flex-grow">
						{profile.name}
					</span>

					<Icon
						icon="mdi:check"
						class="text-accent-500 mx-2 text-lg {i !== activeProfileIndex && 'invisible'}"
					/>

					<div class="mr-1 rounded bg-gray-700 px-1.5 py-0.5 text-xs group-hover:bg-gray-600">
						{profile.modCount}
					</div>

					<Button.Root
						class="rounded p-1 text-gray-400 hover:bg-red-600 hover:text-red-200"
						on:click={(evt) => {
							evt.stopPropagation();
							deleteProfile(i);
							profilesOpen = false;
						}}
					>
						<Icon icon="mdi:delete" />
					</Button.Root>
				</DropdownMenu.Item>
			{/each}

			<DropdownMenu.Item
				class="bg-accent-700 hover:bg-accent-600 flex cursor-default items-center justify-center rounded py-1 text-white"
				on:click={() => (newProfilePopupOpen = true)}
			>
				<Icon icon="mdi:plus" class="mr-1 text-lg" />
				New profile
			</DropdownMenu.Item>
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<Updater />
</div>

<Popup title="Launching {$activeGame?.displayName}..." bind:open={launchGamePopupOpen}>
	<Dialog.Description class="text-gray-400">
		If the game is taking a while to start, it's probably because Steam is starting up.
	</Dialog.Description>
</Popup>

<Popup title="Select game to mod" bind:open={gamesOpen}>
	<GameSelection onSelect={() => (gamesOpen = false)} />
</Popup>

<NewProfilePopup bind:open={newProfilePopupOpen} />
