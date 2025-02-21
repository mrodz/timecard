import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from "@/components/ui/dialog";
import { Button, buttonVariants } from "@/components/ui/button"
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from "@/components/ui/form"
import { Input } from "@/components/ui/input"

import { z } from "zod"
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod"
import { ClockSchema, createUserClock } from "@/lib/api";
import { CurrentUserContext } from "@/pages/Layout";
import { use, useEffect, useState } from "react";
import { cn } from "@/lib/utils";
import { PlusCircle } from "lucide-react";

const createClockSchema = z.object({
	name: z.string({
		required_error: "Name is required",
	}).trim().min(1, {
		message: 'Name must not be empty',
	}).max(64, {
		message: 'Name is too long'
	})
})

type CreateClockProps = {
	onClockCreationStart: (name: string) => void;
	onClockCreated: (clock: ClockSchema) => void;
}

export default function CreateClock(props: CreateClockProps) {
	const user = use(CurrentUserContext)!

	const [open, setOpen] = useState(false)
	const form = useForm<z.infer<typeof createClockSchema>>({
		resolver: zodResolver(createClockSchema),
	})

	useEffect(() => {
		form.formState.isSubmitSuccessful && form.reset()
	}, [form.formState.isSubmitSuccessful, form.reset])

	const submit = async (values: z.infer<typeof createClockSchema>) => {
		props.onClockCreationStart(values.name)
		setOpen(false)

		const clock = await createUserClock({
			userPoolId: user.reactiveUser!.getUsername()!,
			name: values.name,
		})

		props.onClockCreated(clock)
		form.reset()
	}

	return (
		<Dialog open={open} onOpenChange={setOpen}>
			<DialogTrigger className={cn(buttonVariants({ variant: 'ghost' }), 'self-center w-min mx-auto text-slate-100')}>
				Create Clock <PlusCircle />
			</DialogTrigger>
			<DialogContent>
				<DialogHeader>
					<DialogTitle className="text-zinc-950">Create Clock</DialogTitle>
					<DialogDescription>
						Enter required fields below
					</DialogDescription>
				</DialogHeader>

				<Form {...form}>
					<form onSubmit={form.handleSubmit(submit)} className="space-y-8 text-zinc-950">
						<FormField
							control={form.control}
							name="name"
							render={({ field }) => (
								<FormItem>
									<FormLabel>Name</FormLabel>
									<FormControl>
										<Input placeholder="e.g. My Fantastic Clock" {...field} />
									</FormControl>
									<FormDescription>
										A name to keep track of this clock
									</FormDescription>
									<FormMessage />
								</FormItem>
							)}
						/>
						<Button type="submit">Submit</Button>
					</form>
				</Form>
			</DialogContent>
		</Dialog>
	)
}