<script lang="ts">
	import { clearToast, toasts } from '$lib/toast';
	import Icon from '@iconify/svelte';
	import { writeText } from '@tauri-apps/plugin-clipboard-manager';
	import { Button } from 'bits-ui';
	import { expoOut } from 'svelte/easing';
	import { fade, slide } from 'svelte/transition';
</script>

<div
	class="absolute right-0 bottom-0 z-10 flex max-w-[50rem] flex-col items-end justify-end gap-1 p-2 xl:max-w-[90rem]"
>
	{#each $toasts as toast, i}
		<div
			class="flex items-start overflow-hidden rounded-md p-1.5 xl:p-2 xl:text-lg {toast.type ===
			'error'
				? 'bg-red-600'
				: 'bg-accent-600'}"
			in:slide={{ duration: 150, easing: expoOut }}
			out:fade={{ duration: 100 }}
		>
			<div class="mt-auto mr-3 grow overflow-hidden px-2">
				{#if toast.name !== undefined}
					<span class={toast.type === 'error' ? 'text-red-200' : 'text-accent-200'}
						>{toast.name} -</span
					>
				{/if}

				<span class="font-medium break-words text-white">{toast.message}</span>
			</div>

			{#if toast.type === 'error'}
				<Button.Root
					class="rounded-xs p-1 hover:bg-red-500"
					on:click={() => writeText('`' + toast.name + ' - ' + toast.message + '`')}
				>
					<Icon icon="mdi:clipboard-text" class="text-primary-100 text-lg" />
				</Button.Root>
			{/if}

			<Button.Root
				class="rounded-md p-1 {toast.type === 'error' ? 'hover:bg-red-500' : 'hover:bg-accent-500'}"
				on:click={() => clearToast(i)}
			>
				<Icon icon="mdi:close" class="text-primary-100 text-lg" />
			</Button.Root>
		</div>
	{/each}
</div>
