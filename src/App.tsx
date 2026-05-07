/**
 * 占位 App。
 * 后续 feat/ui-design-system 分支会引入 <RanpuLogo /> 与设计系统组件，
 * feat/ui-* 分支会替换为完整的路由 + 守卫。
 */
function App() {
  return (
    <div className="flex min-h-screen flex-col items-center justify-center gap-4 bg-background text-foreground">
      <h1 className="font-serif text-3xl tracking-[3px]">染谱</h1>
      <p className="text-xs uppercase tracking-[2px] text-muted-foreground">
        DYE FORMULA
      </p>
      <p className="mt-8 text-sm text-muted-foreground">
        脚手架就位。下一步：feat/domain-layer。
      </p>
    </div>
  );
}

export default App;
