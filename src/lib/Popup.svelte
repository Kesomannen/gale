<script lang="ts">
	import { Dialog } from 'bits-ui';
	import { fade, scale } from 'svelte/transition';
	import { quartIn, quartOut } from 'svelte/easing';

	export let title: string;
	export let open: boolean;
	export let canClose: boolean = true;
</script>

<Dialog.Root
	bind:open
	closeOnEscape={canClose}
	onOutsideClick={(evt) => {
		if (!canClose) evt.preventDefault();
	}}
>
	<Dialog.Portal>
		<Dialog.Overlay
			class="fixed inset-0 z-50 bg-black/60"
			transition={fade}
			transitionConfig={{ duration: 150 }}
		/>
		<Dialog.Content
			class="
                fixed left-[50%] top-[50%] w-full max-w-[40rem] translate-x-[-50%] translate-y-[-50%]
                z-50 bg-gray-800 rounded-xl p-6 shadow-xl border border-gray-600"
			inTransition={scale}
			inTransitionConfig={{ duration: 200, easing: quartOut, start: 0.5 }}
			outTransition={scale}
			outTransitionConfig={{ duration: 100, easing: quartIn }}
		>
			<Dialog.Title class="w-full text-slate-100 font-bold text-2xl">{title}</Dialog.Title>

			<slot />
		</Dialog.Content>
	</Dialog.Portal>
</Dialog.Root>
