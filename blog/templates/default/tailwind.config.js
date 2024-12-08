/** @type {import('tailwindcss').Config} */
module.exports = {
    content: [
        '../../public/*.html',
        '../../public/**/*.html',
    ],
    theme: {
        extend: {
            maxWidth: {
                'body': '32rem',
                'lg-body': '36rem',
                'xl-body': '42rem',
                '2xl-body': '48rem',
            },

            width: {
                'body': '32rem',
                'lg-body': '68rem',
                'xl-body': '42rem',
                '2xl-body': '48rem',
            }
        },
        fontFamily: {
            sans: [
                '"Proxima Nova"',
                'Roboto',
                'sans-serif',
            ],
            header: [
                '"Segoe UI"',
                'Roboto',
                'sans-serif',
            ],
            logo: [
                '"Fjalla One"',
                'sans-serif',
            ]
        },
    },
}