<script lang="ts">
	import InputField from '$lib/components/InputField.svelte';
	import FormField from '$lib/components/FormField.svelte';
	import Checkbox from '$lib/components/Checkbox.svelte';
	import BigButton from '$lib/components/BigButton.svelte';
	import PathField from '$lib/components/PathField.svelte';
	import Markdown from '$lib/components/Markdown.svelte';

	import { invokeCommand } from '$lib/invoke';
	import type { ModpackArgs } from '$lib/models';
	import { currentProfile } from '$lib/stores';
	import { dialog } from '@tauri-apps/api';
	import { onDestroy } from 'svelte';
	import { fade } from 'svelte/transition';
	import Icon from '@iconify/svelte';
	import ApiKeyPopup, { apiKeyPopupOpen } from '$lib/prefs/ApiKeyPopup.svelte';

	let name: string;
	let author: string;
	let selectedCategories: string[];
	let nsfw: boolean;
	let description: string;
	let readme: string;
	let versionNumber: string;
	let iconPath: string;
	let websiteUrl: string;
	let includeDisabled: boolean;
	let includeFiles: {
		source: string;
		target: string;
		enabled: boolean;
	}[];

	let fetchingArgs = false;
	let loading = false;

	$: {
		$currentProfile;
		refresh();
	}

	async function refresh() {
		fetchingArgs = true;

		let args = await invokeCommand<ModpackArgs>('get_pack_args');

		name = args.name;
		author = args.author;
		selectedCategories = args.categories;
		nsfw = args.nsfw;
		description = args.description;
		readme = args.readme;
		versionNumber = args.versionNumber;
		iconPath = args.iconPath;
		websiteUrl = args.websiteUrl;
		includeDisabled = args.includeDisabled;
		includeFiles = args.includeFiles;

		fetchingArgs = false;
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

		loading = true;
		await invokeCommand('export_pack', { args: args(), dir });
		loading = false;
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

		loading = true;
		try {
			await invokeCommand('upload_pack', { args: args() });
		} finally {
			loading = false;
		}
	}

	function args(): ModpackArgs {
		return {
			name,
			description,
			author,
			categories: selectedCategories,
			nsfw,
			readme,
			versionNumber,
			iconPath,
			websiteUrl,
			includeDisabled,
			includeFiles
		};
	}

	onDestroy(() => {
		invokeCommand('set_pack_args', { args: args() });
	});
</script>

<div class="flex flex-col gap-1.5 py-4 px-6 w-full overflow-y-auto relative">
	{#if fetchingArgs}
		<div
			class="flex items-center justify-center fixed inset-0 text-slate-200 bg-black/40 text-lg"
			transition:fade={{ duration: 50 }}
		>
			<Icon icon="mdi:loading" class="animate-spin mr-4" />
			Loading...
		</div>
	{/if}

	<FormField
		label="Name"
		description="The modpack's name, as shown on Thunderstore. Cannot contain spaces."
		required={true}
	>
		<InputField bind:value={name} placeholder="Enter name..." />
	</FormField>

	<FormField
		label="Author"
		description="The modpack's author, as shown on Thunderstore."
		required={true}
	>
		<InputField bind:value={author} placeholder="Enter author..." />
	</FormField>

	<FormField label="Description" description="A short description of the modpack." required={true}>
		<InputField bind:value={description} placeholder="Enter description..." />
	</FormField>

	<!--
    <FormField
      label="Categories"
      description="The modpack's name, as shown on Thunderstore. Cannot contain spaces."
    >
      {#if selectedCategories}
        <Dropdown
          avoidCollisions={false}
          items={$categories}
          bind:selected={selectedCategories}
          multiple={true}
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
                    <span class="truncate overflow-hidden">{category}</span>

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
    -->

	<FormField
		label="Version"
		description="
        The modpack's version number, in the format of X.Y.Z.
        Should preferably follow semantic versioning.
      "
		required={true}
	>
		<InputField bind:value={versionNumber} placeholder="Enter name..." />
	</FormField>

	<FormField label="Website" description="A link to a website of your choosing. Optional.">
		<InputField bind:value={websiteUrl} placeholder="Enter website URL..." />
	</FormField>

	<FormField
		label="Icon"
		description="
        Path to the modpack's icon. This is automatically resized to 256x256 pixels, so
        it's recommended to be a square image to avoid stretching or squishing.
      "
		required={true}
	>
		<PathField icon="mdi:file-image" onClick={browseIcon} value={iconPath} />
	</FormField>

	<FormField
		label="Readme"
		description="A longer description of the modpack, which supports markdown formatting (similarly to Discord messages)."
		required={true}
	>
		<textarea
			class="w-full h-32 px-3 py-2 rounded-lg bg-gray-900 placeholder-slate-400 text-slate-200
            border-slate-500 border-opacity-0 border hover:border-opacity-100"
			placeholder="Enter readme..."
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
				<div class="border border-gray-900 text-slate-300 mt-1">
					{#each includeFiles as { source, enabled }, i}
						<div class="flex items-center justify-between odd:bg-gray-900 px-2 py-1">
							{source}

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
		<BigButton color="gray" disabled={loading} on:click={exportToFile}>Export to file</BigButton>
		<BigButton color="green" disabled={loading} on:click={uploadToThunderstore}
			>Publish to Thunderstore</BigButton
		>
	</div>
</div>

<ApiKeyPopup />
