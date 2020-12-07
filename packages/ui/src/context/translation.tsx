import { FC, useMemo } from 'react'
import { createContext, useContext } from 'react'
import { Link } from 'react-router-dom'
import { ErrorCode } from '@riezler/auth-sdk'

type DisclaimerProps = { tos: string, privacy: string }

type Translations = {
	signin: {
		title: string;
		label: string;
		info: string;
	};

	signup: {
		title: string;
		label: string;
		info: string;
	};

	email: {
		label: string;
	};

	password: {
		label: string;
		title: string;
		forgot: string;
	};

	label: {
		email: string;
		password: string;
	};

	Disclaimer: FC<DisclaimerProps>;

	error: {
		password_length: string;
		invalid_email_password: string;
		generic: string;
		not_allowed: string;
	};
}

export let DefaultTranslation = {
	signin: {
		title: 'Welcome Back.',
		label: 'Sign In',
		info: 'Already have an account?'
	},

	signup: {
		title: 'Create an account',
		label: 'Sign Up',
		info: `Don't have an account?`
	},

	email: {
		label: 'Authentication Link'
	},

	label: {
		email: 'Email',
		password: 'Password',
	},

	password: {
		label: 'Email and Password',
		title: 'Email and Password',
		forgot: 'Forgot Password?',
	},

	Disclaimer: ({ tos, privacy }: DisclaimerProps) => {
		return (
			<small>
				By continuing you agree to our <Link to={tos}>Terms of Use</Link> and <Link to={privacy}>Privacy Policy</Link>.
			</small>
		)
	},

	error: {
		password_length: 'Your password sould be at least 8 characters long',
		invalid_email_password: 'Invalid Email or Password',
		generic: 'Something went wrong',
		not_allowed: 'Not Allowed'
	}
}

export let Translation = createContext(DefaultTranslation)

export function useTranslation(): Translations {
	return useContext(Translation)
}

export function useError(code: ErrorCode | null): string | null {
	let t = useTranslation()
	return useMemo(() => {
		if (code === null) {
			return null
		}

		switch(code) {
			case ErrorCode.AuthPasswordLength:
				return t.error.password_length

			case ErrorCode.InvalidEmailPassword:
				return t.error.invalid_email_password

			case ErrorCode.NotAllowed:
				return t.error.not_allowed

			default:
				return t.error.generic
		}
	}, [code, t])
}