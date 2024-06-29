<script lang="ts">
	import InputField from '$lib/components/InputField.svelte';
	import FormField from '$lib/components/FormField.svelte';
	import Checkbox from '$lib/components/Checkbox.svelte';
	import BigButton from '$lib/components/BigButton.svelte';
	import PathField from '$lib/components/PathField.svelte';
	import Markdown from '$lib/components/Markdown.svelte';
	import Dropdown from '$lib/components/Dropdown.svelte';
	import Link from '$lib/components/Link.svelte';
	import ApiKeyPopup, { apiKeyPopupOpen } from '$lib/prefs/ApiKeyPopup.svelte';

	import { invokeCommand } from '$lib/invoke';
	import type { ModpackArgs, PackageCategory } from '$lib/models';
	import { activeProfile, activeGame, categories } from '$lib/stores';
	import { dialog } from '@tauri-apps/api';
	import { onDestroy } from 'svelte';
	import { fade } from 'svelte/transition';
	import Icon from '@iconify/svelte';

	import { Button, Dialog, Select } from 'bits-ui';
	import Popup from '$lib/components/Popup.svelte';

	const URL_PATTERN =
		'[Hh][Tt][Tt][Pp][Ss]?://(?:(?:[a-zA-Z\u00a1-\uffff0-9]+-?)*[a-zA-Z\u00a1-\uffff0-9]+)(?:.(?:[a-zA-Z\u00a1-\uffff0-9]+-?)*[a-zA-Z\u00a1-\uffff0-9]+)*(?:.(?:[a-zA-Z\u00a1-\uffff]{2,}))(?::d{2,5})?(?:/[^s]*)?';

	let name: string;
	let author: string;
	let selectedCategories: PackageCategory[];
	let nsfw: boolean;
	let description: string;
	let readme: string;
	let versionNumber: string;
	let iconPath: string;
	let websiteUrl: string;
	let includeDisabled: boolean;
	let includeFiles: {
		path: string;
		enabled: boolean;
	}[];

	let donePopupOpen = false;
	let loading: string | null = null;

	$: {
		$activeProfile;
		refresh();
	}

  // make sure 'Modpacks' category is always selected
	$: if (
		selectedCategories &&
		!selectedCategories.some((category) => category?.name === 'Modpacks')
	) {
		selectedCategories = [
			$categories.find((category) => category.name === 'Modpacks')!,
			...selectedCategories
		];
	}

	async function refresh() {
		loading = 'Loading...';

		let args = await invokeCommand<ModpackArgs>('get_pack_args');

		name = args.name;
		author = args.author;
		nsfw = args.nsfw;
		description = args.description;
		selectedCategories = args.categories.map(
			(selected) => $categories.find((category) => category.slug === selected)!
		);
		readme = args.readme;
		versionNumber = args.versionNumber;
		iconPath = args.iconPath;
		websiteUrl = args.websiteUrl;
		includeDisabled = args.includeDisabled;
		includeFiles = args.includeFiles;

		loading = null;
	}

	async function browseIcon() {
		let path = await dialog.open({
			defaultPath: iconPath.length > 0 ? iconPath : undefined,
			title: 'Select modpack icon',
			filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'gif'] }]
		});

		if (!path) return;
		iconPath = path as string;
	}

	async function exportToFile() {
		let dir = await dialog.open({
			title: 'Choose directory to save modpack',
			defaultPath: `${name}.zip`,
			directory: true
		});

		if (!dir) return;

		loading = 'Exporting modpack to file...';
		try {
			await invokeCommand('export_pack', { args: args(), dir });
		} finally {
			loading = null;
		}
	}

	async function uploadToThunderstore() {
		let hasToken = await invokeCommand('has_thunderstore_token');

		if (!hasToken) {
			$apiKeyPopupOpen = true;

			await new Promise<void>((resolve) => {
				const interval = setInterval(() => {
					if (!$apiKeyPopupOpen) {
						clearInterval(interval);
						resolve();
					}
				}, 100);

				return () => clearInterval(interval);
			});

			hasToken = await invokeCommand('has_thunderstore_token');

			if (!hasToken) return;
		}

		loading = 'Uploading modpack to Thunderstore...';
		try {
			await invokeCommand('upload_pack', { args: args() });
			donePopupOpen = true;
		} finally {
			loading = null;
		}
	}

	function args(): ModpackArgs {
		return {
			name,
			description,
			author,
			nsfw,
			readme,
			versionNumber,
			iconPath,
			websiteUrl,
			includeDisabled,
			includeFiles,
			categories: selectedCategories.map((c) => c.slug)
		};
	}

	onDestroy(() => {
		invokeCommand('set_pack_args', { args: args() });
	});
</script>

<form class="flex flex-col gap-1.5 py-4 px-6 w-full overflow-y-auto relative">
	{#if loading}
		<div
			class="flex items-center justify-center fixed inset-0 text-slate-200 bg-black/40 text-lg"
			transition:fade={{ duration: 50 }}
		>
			<Icon icon="mdi:loading" class="animate-spin mr-4" />
			{loading}
		</div>
	{/if}

	<FormField
		label="Name"
		description="The name of the modpack, as shown on Thunderstore. 
			        Make sure this stays consistent between updates.
			        Cannot contain spaces or hyphens."
		required={true}
	>
		<InputField
			bind:value={name}
			placeholder="Enter name..."
			required={true}
			pattern="^[a-zA-Z0-9_]+$"
		/>
	</FormField>

	<FormField
		label="Author"
		description="The author of the modpack, which should be the name of your Thunderstore team."
		required={true}
	>
		<InputField bind:value={author} placeholder="Enter author..." required={true} />
	</FormField>

	<FormField label="Description" description="A short description of the modpack." required={true}>
		<InputField
			bind:value={description}
			placeholder="Enter description..."
			required={true}
			maxlength={250}
		/>
	</FormField>

	<FormField
		label="Categories"
		description="The categories that the modpack belongs to. 'Modpacks' is always included."
	>
		{#if selectedCategories}
			<Dropdown
				avoidCollisions={false}
				items={$categories}
				bind:selected={selectedCategories}
				multiple={true}
				getLabel={(category) => category.name}
			>
				<Select.Trigger
					let:open
					slot="trigger"
					class="flex items-center w-full bg-gray-900 rounded-lg pl-1 pr-3 py-1 overflow-hidden
                  border border-gray-500 border-opacity-0 hover:border-opacity-100"
				>
					{#if selectedCategories.length === 0}
						<span class="text-slate-400 truncate pl-2">Select categories...</span>
					{:else}
						<div class="flex flex-wrap gap-1">
							{#each selectedCategories as category}
								<div class="bg-gray-800 text-slate-200 rounded-md pl-3 pr-1 py-1 text-sm">
									<span class="truncate overflow-hidden">{category.name}</span>

									<Button.Root
										class="px-1.5 ml-1 rounded-md hover:bg-gray-700"
										on:click={(evt) => {
											evt.stopPropagation();
											selectedCategories = selectedCategories.filter((c) => c !== category);
										}}
									>
										x
									</Button.Root>
								</div>
							{/each}
						</div>
					{/if}
					<Icon
						class="text-slate-400 text-xl transition-all duration-100 ease-out ml-auto flex-shrink-0
                transform origin-center {open ? 'rotate-180' : 'rotate-0'}"
						icon="mdi:chevron-down"
					/>
				</Select.Trigger>
			</Dropdown>
		{/if}
	</FormField>

	<FormField
		label="Version"
		description="The version number of the modpack, in the format of X.Y.Z.
			           You cannot publish with the same version number twice."
		required={true}
	>
		<InputField
			bind:value={versionNumber}
			placeholder="Enter version number..."
			required={true}
			pattern="^\d+\.\d+\.\d+$"
		/>
	</FormField>

	<FormField label="Website" description="The URL of a website of your choosing. Optional.">
		<InputField bind:value={websiteUrl} placeholder="Enter website URL..." pattern={URL_PATTERN} />
	</FormField>

	<FormField
		label="Icon"
		description="Path to the icon of the modpack. This is automatically resized to 256x256 pixels, so
                 it's recommended to be a square image to avoid stretching or squishing."
		required={true}
	>
		<PathField icon="mdi:file-image" onClick={browseIcon} value={iconPath} />
	</FormField>

	<FormField
		label="Readme"
		description="A longer description of the modpack, which supports markdown formatting 
                 (similarly to Discord messages)."
		required={true}
	>
		<textarea
			class="w-full h-32 px-3 py-2 rounded-lg bg-gray-900 placeholder-slate-400
           text-slate-300 hover:text-slate-200
			     valid:focus:border-green-400 invalid:border-red-500 focus:outline-none
			       border border-slate-500 border-opacity-0 hover:border-opacity-100"
			placeholder="Enter readme..."
			required={true}
			bind:value={readme}
		/>

		<details class="mt-1">
			<summary class="text-sm text-slate-300 cursor-pointer">Preview</summary>
			<Markdown class="px-4 mt-1 bg-gray-900 rounded-lg" source={readme} />
		</details>
	</FormField>

	<FormField
		label="Include files ({includeFiles?.filter(({ enabled }) => enabled)
			.length}/{includeFiles?.length})"
		description="Choose which config files to include in the modpack."
	>
		<details>
			{#if includeFiles}
				<summary class="text-sm text-slate-300 cursor-pointer">Show list</summary>
				<div class="mt-1 overflow-hidden rounded-md">
					{#each includeFiles as { path, enabled }, i}
						<div
							class="flex items-center justify-between py-1.5 gap-2 px-3 text-slate-300
                    			 odd:bg-gray-900 hover:bg-gray-700"
						>
							{path}

							<Checkbox
								value={enabled}
								onValueChanged={(newValue) => (includeFiles[i].enabled = newValue)}
							/>
						</div>
					{/each}
				</div>
			{/if}
		</details>
	</FormField>

	<div class="flex items-center text-lg font-medium text-slate-200 mt-1">
		<span class="max-w-96 flex-grow">Contains NSFW content</span>

		<Checkbox bind:value={nsfw} />
	</div>

	<div class="flex items-center text-lg font-medium text-slate-200">
		<span class="max-w-96 flex-grow">Include disabled mods</span>

		<Checkbox bind:value={includeDisabled} />
	</div>

	<div class="flex justify-end gap-2 mt-3">
		<BigButton color="gray" on:click={exportToFile}>Export to file</BigButton>
		<BigButton color="green" on:click={uploadToThunderstore}>Publish on Thunderstore</BigButton>
	</div>
</form>

<ApiKeyPopup />

<Popup bind:open={donePopupOpen} title="Modpack upload complete">
	<Dialog.Description class="text-slate-300">
		{name}
		{versionNumber} has successfully been published on Thunderstore!
		<Link href="https://thunderstore.io/c/{$activeGame?.id}/p/{author}/{name}"
			>Click here to view its page on the website</Link
		>.
	</Dialog.Description>

	<div class="mt-2 text-slate-400 text-sm">
		The changes may take up to an hour to appear in Gale and other mod managers.
		<br />
		To publish a new update, increment the version number and publish the modpack again.
	</div>
</Popup>
