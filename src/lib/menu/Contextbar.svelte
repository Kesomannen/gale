<script lang="ts">
	import { confirm } from '@tauri-apps/plugin-dialog';

	import Popup from '$lib/components/Popup.svelte';
	import NewProfilePopup from '$lib/menu/NewProfilePopup.svelte';

	import {
		activeProfileId,
		activeGame,
		activeProfile,
		profiles,
		refreshProfiles,
		setActiveProfile
	} from '$lib/stores';
	import { invokeCommand } from '$lib/invoke';

	import Icon from '@iconify/svelte';
	import { Dialog, DropdownMenu } from 'bits-ui';
	import GameSelection from '$lib/menu/GameSelection.svelte';
	import Updater from './Updater.svelte';
	import { pushInfoToast } from '$lib/toast';
	import Syncer from './Syncer.svelte';

	let launchGamePopupOpen = $state(false);
	let newProfilePopupOpen = $state(false);

	let gamesOpen = $state(false);
	let profilesOpen = $state(false);

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

	function launchGame() {
		invokeCommand('launch_game');
		launchGamePopupOpen = true;
	}
</script>

<div class="border-primary-600 bg-primary-900 flex h-12 shrink-0 flex-row border-t border-b">
	<div
		class="text-accent-400 hover:text-accent-400 border-primary-600 hover:bg-primary-800 shrink-0 border-r pr-8 pl-6"
	>
		<button class="flex h-full items-center font-semibold" onclick={launchGame}>
			<Icon icon="mdi:play-circle" class="mr-2 text-xl" />
			Launch game
		</button>
	</div>

	<button
		onclick={() => (gamesOpen = !gamesOpen)}
		class="group border-primary-600 text-primary-300 group-hover:text-primary-200 hover:bg-primary-800 flex shrink-0 items-center justify-between border-r pr-4 pl-2 font-semibold"
	>
		<img
			src="games/{$activeGame?.slug}.webp"
			class="mr-2 max-h-8 max-w-8 rounded-sm"
			alt={$activeGame?.name}
		/>

		{$activeGame?.name}

		<Icon
			icon="mdi:menu"
			class="text-primary-300 group-hover:text-primary-200 ml-6 shrink-0 text-xl"
		/>
	</button>

	<DropdownMenu.Root bind:open={profilesOpen}>
		<DropdownMenu.Trigger
			class="group border-primary-600 text-primary-300 group-hover:text-primary-200 hover:bg-primary-800 flex min-w-40 shrink items-center border-r pr-4 pl-6"
		>
			<span class="mr-auto shrink truncate font-semibold">
				{$activeProfile?.name}
			</span>

			<div
				class="bg-primary-800 group-hover:bg-primary-700 mr-2 ml-6 rounded-sm px-2 py-0.5 text-sm font-medium"
			>
				{$activeProfile?.modCount}
			</div>

			<Icon
				icon="mdi:expand-more"
				class={[
					profilesOpen ? 'rotate-180' : 'rotate-0',
					'shrink-0 origin-center transform text-xl transition-all'
				]}
			/>
		</DropdownMenu.Trigger>
		<DropdownMenu.Content
			class="border-primary-600 bg-primary-800 flex max-h-[80lvh] min-w-40 flex-col gap-0.5 overflow-y-auto rounded-b-lg border p-1 shadow-xl"
		>
			{#each profiles as profile, index}
				<DropdownMenu.Item
					class={[
						profile.id == activeProfileId
							? 'text-primary-300 hover:text-primary-200 font-medium'
							: 'text-primary-400 hover:text-primary-300',
						'group hover:bg-primary-700 flex cursor-default items-center rounded-md py-1 pr-1 pl-3 text-left'
					]}
					onclick={() => {
						setActiveProfile(index);
						profilesOpen = false;
					}}
				>
					{#if profile.sync !== null}
						<Icon icon="mdi:cloud" class="mr-2" />
					{/if}

					<span class="mr-3 grow">
						{profile.name}
					</span>

					<Icon
						icon="mdi:check"
						class="text-accent-500 mx-2 text-lg {profile.id !== activeProfileId && 'invisible'}"
					/>

					<div
						class="bg-primary-700 group-hover:bg-primary-600 mr-1 rounded-sm px-1.5 py-0.5 text-xs"
					>
						{profile.modCount}
					</div>

					<button
						class="text-primary-400 rounded-sm p-1 hover:bg-red-600 hover:text-red-200"
						onclick={(evt) => {
							evt.stopPropagation();
							deleteProfile(index);
							profilesOpen = false;
						}}
					>
						<Icon icon="mdi:delete" />
					</button>
				</DropdownMenu.Item>
			{/each}

			<DropdownMenu.Item
				class="bg-accent-700 hover:bg-accent-600 flex cursor-default items-center justify-center rounded-sm py-1 text-white"
				onclick={() => (newProfilePopupOpen = true)}
			>
				<Icon icon="mdi:plus" class="mr-1 text-lg" />
				New profile
			</DropdownMenu.Item>
		</DropdownMenu.Content>
	</DropdownMenu.Root>

	<Syncer />
	<Updater />
</div>

<Popup title="Launching {$activeGame?.name}..." bind:open={launchGamePopupOpen}>
	<Dialog.Description class="text-primary-400">
		If the game is taking a while to start, it's probably because Steam is starting up.
	</Dialog.Description>
</Popup>

<Popup title="Select game to mod" bind:open={gamesOpen}>
	<GameSelection onselect={() => (gamesOpen = false)} />
</Popup>

<NewProfilePopup bind:open={newProfilePopupOpen} />
