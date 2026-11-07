import Header from "@/layouts/components/header";
import Footer from "@/layouts/components/footer";
import type { ComponentChildren } from "preact";

interface ContainerProps {
  children?: ComponentChildren;
}

export default function DefaultLayout({ children }: ContainerProps) {
  return (
    <>
      <Header />
      <div class="container">{children}</div>
      <Footer />
    </>
  );
}
