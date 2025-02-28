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
	import { pushInfoToast } from '$lib/toast';

	let launchGamePopupOpen = false;
	let newProfilePopupOpen = false;

	let gamesOpen = false;
	let profilesOpen = false;

	function deleteProfile(index: number) {
		confirm(`Are you sure you want to delete ${profiles[index].name}?`).then(async (result) => {
			if (result) {
				await invokeCommand('delete_profile', { index });

				pushInfoToast({
					message: `Deleted profile ${profiles[index].name}.`
				});

				refreshProfiles();
			}
		});
	}

	function launchGame(vanilla: boolean) {
		invokeCommand('launch_game', { vanilla });
		launchGamePopupOpen = true;
	}
</script>

<div class="flex h-12 shrink-0 flex-row border-t border-b border-slate-600 bg-slate-900">
	<div
		class="text-accent-400 hover:text-accent-400 shrink-0 border-r border-slate-600 pr-8 pl-6 hover:bg-slate-800"
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
		class="group flex shrink-0 cursor-default items-center justify-between border-r border-slate-600 pr-4 pl-2 font-semibold text-slate-300 group-hover:text-slate-200 hover:bg-slate-800"
	>
		{#if $activeGame}
			<img
				src="games/{$activeGame.slug}.webp"
				class="mr-2 max-h-8 max-w-8 rounded-sm"
				alt={$activeGame.name}
			/>

			{$activeGame.name}
		{:else}
			Loading...
		{/if}

		<Icon
			icon="mdi:menu"
			class="ml-6 shrink-0 text-xl text-slate-300 transition-all group-hover:text-slate-200"
		/>
	</Button.Root>

	<DropdownMenu.Root bind:open={profilesOpen}>
		<DropdownMenu.Trigger
			class="group flex min-w-40 shrink cursor-default items-center border-r border-slate-600 pr-4 
						pl-6 text-slate-300 group-hover:text-slate-200 hover:bg-slate-800"
		>
			<span class="mr-auto shrink truncate font-semibold">
				{$activeProfile?.name}
			</span>

			<div
				class="mr-2 ml-6 rounded-sm bg-slate-800 px-2 py-0.5 text-sm font-medium group-hover:bg-slate-700"
			>
				{$activeProfile?.modCount}
			</div>

			<Icon
				icon="mdi:expand-more"
				class="shrink-0 origin-center transform text-xl transition-all {profilesOpen
					? 'rotate-180'
					: 'rotate-0'}"
			/>
		</DropdownMenu.Trigger>
		<DropdownMenu.Content
			class="flex max-h-[80lvh] min-w-40 flex-col gap-0.5 overflow-y-auto rounded-b-lg border border-slate-600 bg-slate-800 p-1 shadow-xl"
			{...dropTransition}
		>
			{#each profiles as profile, i}
				<DropdownMenu.Item
					class="group flex cursor-default items-center rounded-md py-1 pr-1 pl-3 text-left hover:bg-slate-700
						{i == activeProfileIndex
						? 'font-medium text-slate-300 hover:text-slate-200'
						: 'text-slate-400 hover:text-slate-300'}"
					on:click={() => {
						setActiveProfile(i);
						profilesOpen = false;
					}}
				>
					<span class="mr-3 grow">
						{profile.name}
					</span>

					<Icon
						icon="mdi:check"
						class="text-accent-500 mx-2 text-lg {i !== activeProfileIndex && 'invisible'}"
					/>

					<div class="mr-1 rounded-sm bg-slate-700 px-1.5 py-0.5 text-xs group-hover:bg-slate-600">
						{profile.modCount}
					</div>

					<Button.Root
						class="rounded-sm p-1 text-slate-400 hover:bg-red-600 hover:text-red-200"
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
				class="bg-accent-700 hover:bg-accent-600 flex cursor-default items-center justify-center rounded-sm py-1 text-white"
				on:click={() => (newProfilePopupOpen = true)}
			>
				<Icon icon="mdi:plus" class="mr-1 text-lg" />
				New profile
			</DropdownMenu.Item>
		</DropdownMenu.Content>
	</DropdownMenu.Root>

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

<NewProfilePopup bind:open={newProfilePopupOpen} />
