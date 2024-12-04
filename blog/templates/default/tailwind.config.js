/** @type {import('tailwindcss').Config} */
module.exports = {
    content: [
        '../../public/*.html',
        '../../public/**/*.html',
    ],
    theme: {
        fontFamily: {
            sans: [
                '"Proxima Nova"',
                'Roboto',
                'sans-serif',
            ],
            header: ['"Segoe UI"',
                'Roboto',
                'sans-serif',
            ]
        },
    },
}