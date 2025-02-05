import { Link } from "react-router-dom";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "./ui/accordion";
import { buttonVariants } from "./ui/button";
import { Component } from "react";
import { getAuthUrl, InvalidCodeRedirectError } from "@/lib/useAuth";
import { Card } from "./ui/card";

export default class AuthErrorBoundary extends Component<{ children: any }, { hasError: boolean, error: any }> {
	constructor(props: { children: any }) {
		super(props)
		this.state = { error: undefined, hasError: false }
	}

	static getDerivedStateFromError(error: any) {
		return { hasError: true, error };
	}

	render() {
		const GenericError = () => (
			<Accordion type="single" collapsible className="my-4 md:my-8 text-left">
				<AccordionItem value="item-1">
					<AccordionTrigger className={buttonVariants({ variant: 'outline' })}>See Stack Trace</AccordionTrigger>
					<AccordionContent>
						<pre className="overflow-x-scroll">
							{this.state.error?.trace ?? JSON.stringify(this.state.error)}
						</pre>
					</AccordionContent>
				</AccordionItem>
			</Accordion>
		)

		const CodeRedirectError = () => (
			<>
				<p className="my-4 md:my-8">
					This code/page is invalid. Please sign in again.
				</p>
				<Link className={buttonVariants({ variant: 'secondary' })} to={getAuthUrl()}>Sign In</Link>
			</>
		)

		if (this.state.hasError) {
			console.trace((this.state?.error as any) instanceof InvalidCodeRedirectError);
			console.trace(this.state?.error);

			return (
				<div className="bg-red-400 min-h-screen flex flex-col justify-center items-center">
					<Card className="inline-block w-4/5 md:w-1/3 p-4 md:p-8">
						<p>
							An unexpected error occurred ⚠️
						</p>
						{!!this.state.error && this.state?.error as any instanceof InvalidCodeRedirectError
							? <CodeRedirectError />
							: <GenericError />
						}
					</Card>
				</div >
			)
		}
		return this.props.children;
	}
}