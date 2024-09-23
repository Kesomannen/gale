<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Select } from 'bits-ui';
	import { expoOut, quartIn, quartOut, quintOut } from 'svelte/easing';
	import { fly, scale, slide } from 'svelte/transition';
</script>

<div class="space-y-5 p-6">
	<div class="flex">
		<button
			class="flex items-center gap-1 rounded-lg py-1.5 px-3 bg-green-600 text-white hover:bg-green-700"
		>
			<Icon icon="material-symbols:download" class="text-lg" />
			Install
		</button>
	</div>

	<div class="flex">
		<button
			class="flex items-center gap-1 rounded-full py-1.5 px-5 bg-green-600 text-black hover:bg-green-500 transition-colors"
		>
			<Icon icon="material-symbols:download" class="text-lg" />
			<span class="font-bold">Install</span>
		</button>
	</div>

	<div class="flex">
		<button
			class="flex items-center gap-1 rounded px-4 py-2 bg-green-600 text-white hover:bg-green-700 transition-transform hover:-translate-y-0.5"
		>
			<Icon icon="material-symbols:download" class="text-lg" />
			<span>Install</span>
		</button>
	</div>

	<div class="flex gap-2 p-8 ">
		<button
			class="flex items-center gap-1 rounded-lg px-3 py-1.5 bg-green-700 text-white transition-all hover:-translate-y-0.5 border-2 border-green-500 hover:border-green-400 active:translate-y-0 active:bg-green-800 active:border-green-600 duration-100"
		>
			<Icon icon="material-symbols:download" class="text-lg" />
			<span>Install</span>
		</button>

		<button
			class="flex items-center gap-1 rounded-lg px-3 py-1.5 bg-red-700 text-white transition-all hover:-translate-y-0.5 border-2 border-red-500 hover:border-red-400"
		>
			<Icon icon="material-symbols:delete" />
			<span>Uninstall</span>
		</button>

		<button
			class="flex items-center gap-1 rounded-lg px-3 py-1.5 bg-gray-700 text-gray-300 transition-all enabled:hover:-translate-y-0.5 border-2 border-gray-500 enabled:hover:border-gray-400 disabled:opacity-65 disabled:border-gray-600 cursor-not-allowed"
			disabled
		>
			<Icon icon="material-symbols:delete" class="text-lg" />
			<span>Uninstall</span>
		</button>

		<Select.Root>
			<Select.Trigger
				class="flex items-center gap-2 rounded-lg px-3 py-1.5 bg-gray-700 border-2 border-gray-600 text-gray-100 w-40  active:border-gray-500 duration-100"
			>
				<Icon icon="material-symbols:sort" />
				<Select.Value />
			</Select.Trigger>
			<Select.Content
				inTransition={fly}
				inTransitionConfig={{ duration: 100, y: -10, easing: quartOut }}
        outTransition={fly}
        outTransitionConfig={{ duration: 50, y: -20, easing: quartIn }}
				sideOffset={-4}
				class="flex flex-col bg-gray-700 rounded-b-lg p-1 border-b-2 border-x-2 border-gray-600"
			>
				{#each ['Relevance', 'Last updated', 'Downloads'] as item}
					<Select.Item
						value={item}
						class="text-gray-300 hover:text-gray-100 px-2 py-1 hover:bg-gray-600 cursor-pointer rounded"
					>
						{item}
					</Select.Item>
				{/each}
			</Select.Content>
		</Select.Root>

    <Select.Root>
			<Select.Trigger
				class="flex items-center gap-2 rounded-lg px-3 py-1.5 bg-gray-700 border-2 border-gray-600 hover:border-gray-500 text-gray-100 w-40  duration-100"
			>
				<Icon icon="material-symbols:settings" />
				<Select.Value />
			</Select.Trigger>
			<Select.Content
				inTransition={fly}
				inTransitionConfig={{ duration: 100, y: -10, easing: quartOut }}
        outTransition={fly}
        outTransitionConfig={{ duration: 50, y: -20, easing: quartIn }}
				sideOffset={-5}
				class="flex flex-col bg-gray-700 rounded-b-lg p-1 border-b-2 border-x-2 border-gray-600"
			>
				{#each ['New profile', 'Overwrite profile'] as item}
					<Select.Item
						value={item}
						class="text-gray-300 hover:text-gray-100 px-2 py-1 hover:bg-gray-600 cursor-pointer rounded"
					>
						{item}
					</Select.Item>
				{/each}
			</Select.Content>
		</Select.Root>
	</div>
</div>
