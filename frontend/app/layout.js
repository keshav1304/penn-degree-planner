import "./globals.css";

export const metadata = {
  title: "Penn Degree Planner",
  description: "Plan your Penn degree requirements and schedule across multiple programs",
};

export default function RootLayout({ children }) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
