<script lang="ts">
	import GameSelection from '$lib/components/GameSelection.svelte';
	import Popup from '$lib/components/Popup.svelte';
	import TabsMenu from '$lib/components/TabsMenu.svelte';
	import Checkbox from '$lib/components/Checkbox.svelte';
	import BigButton from '$lib/components/BigButton.svelte';

  import { refreshProfiles } from '$lib/profile';

	import Icon from '@iconify/svelte';

	import { fade } from 'svelte/transition';

	import { invokeCommand } from '$lib/invoke';
	import { listen } from '@tauri-apps/api/event';

	interface ImportData {
		r2modman?: {
			path: string;
			profiles: string[];
			include: boolean[];
		};
		thunderstore?: {
			path: string;
			profiles: string[];
			include: boolean[];
		};
	}

	export let open: boolean;

	let loading = false;
	let loadingText = '';

	let stage: 'gameSelect' | 'importProfiles' | 'end' = 'gameSelect';

	let importData: ImportData = {
		r2modman: undefined,
		thunderstore: undefined
	};
	let importFrom: 'r2modman' | 'thunderstore' = 'r2modman';

	$: title = stage === 'importProfiles' ? 'Import profiles' : 'Welcome to Gale!';
	$: importText =
		importData.r2modman && importData.thunderstore
			? 'r2modman or Thunderstore Mod Manager'
			: importData.r2modman
				? 'r2modman'
				: 'Thunderstore Mod Manager';

	async function onSelectGame() {
		loading = true;
		let result = await invokeCommand<ImportData>('get_r2modman_info');

		if (!result.r2modman && !result.thunderstore) {
			stage = 'end';
			loading = false;
			return;
		}

		importData = result;

		if (result.r2modman) {
			result.r2modman.include = result.r2modman.profiles.map(() => true);
			importFrom = 'r2modman';
		}

		if (result.thunderstore) {
			result.thunderstore.include = result.thunderstore.profiles.map(() => true);
			importFrom = 'thunderstore';
		}

		stage = 'importProfiles';
		loading = false;
	}

	async function importProfiles() {
		let data = importData[importFrom];

		if (!data) {
			return;
		}

		loading = true;

		let unlisten = await listen<string>('transfer_update', (evt) => {
			loadingText = evt.payload;
		});

    try {
		  await invokeCommand('import_r2modman', { path: data.path, include: data.include });
      refreshProfiles();
      stage = 'end';
    } finally {
			unlisten();
      loading = false;
    }
  }
</script>

<Popup {title} canClose={!loading} bind:open maxWidth="[55%]">
	{#if loading}
		<div
			class="inset-0 absolute z-50 flex flex-col gap-3 items-center justify-center bg-black/60"
			transition:fade={{ duration: 50 }}
		>
			<Icon icon="mdi:loading" class="text-6xl text-slate-600 animate-spin" />
			<div class="text-lg text-slate-400">{loadingText}</div>
		</div>
	{/if}

	<div class="text-slate-300">
		{#if stage === 'gameSelect'}
			<h2 class="text-lg">To get started, select a game to mod:</h2>
			<GameSelection onSelect={onSelectGame} />
		{:else if stage === 'importProfiles' && importData}
			<p>
				You can automatically transfer profiles from {importText} to Gale.
			</p>

			<p class="mt-1">
				The process may take a couple of minutes, depending on how many mods and profiles there are
				to import. It will also import configs and cached mods.
			</p>

			<p class="mt-1">
				You can always import profiles later by going to <b>Import > ...from r2modman</b>
			</p>

			<h3 class="text-lg text-slate-200 font-semibold mt-3">Choose profiles to import</h3>

			{#if importData.r2modman && importData.thunderstore}
				<TabsMenu
					bind:value={importFrom}
					options={[
						{ value: 'r2modman', label: 'r2modman' },
						{ value: 'thunderstore', label: 'Thunderstore Mod Manager' }
					]}
				/>
			{/if}

			<div class="flex flex-col h-72 overflow-y-auto">
				{#each importData[importFrom].profiles as profile, index}
					<div class="flex items-center justify-between py-1">
						{profile}

						<Checkbox
							value={importData[importFrom].include[index]}
							onValueChanged={(value) => (importData[importFrom].include[index] = value)}
						/>
					</div>
				{/each}
			</div>

			<div class="flex mt-2 gap-1.5">
				<BigButton color="gray" onClick={() => (stage = 'gameSelect')}>Back</BigButton>
				<div class="flex-grow" />
				<BigButton color="gray" onClick={() => (stage = 'end')}>Skip</BigButton>
				<BigButton color="green" onClick={importProfiles}>Import</BigButton>
			</div>
		{:else if stage === 'end'}
			<p>That's it, you're all set up to start modding!</p>

			<p>
				If you have any questions or need help, feel free to ask in the <a
					href="https://discord.gg/7v8vYR9"
					target="_blank"
					class="text-green-400 hover:underline">Discord server</a
				>.
			</p>
		{/if}
	</div>
</Popup>
