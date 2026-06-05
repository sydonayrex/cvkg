import os
def remove_pub_crate_in_traits(filepath):
    if not os.path.exists(filepath): return
    with open(filepath, 'r') as f:
        lines = f.readlines()
    in_trait = False
    for i in range(len(lines)):
        if 'impl cvkg_core::' in lines[i] or 'impl Drop for ' in lines[i] or 'impl FrameRenderer for ' in lines[i] or 'impl ElapsedTime for ' in lines[i]:
            in_trait = True
        elif in_trait and lines[i].startswith('}'):
            in_trait = False
        
        if in_trait and 'pub(crate) fn' in lines[i]:
            lines[i] = lines[i].replace('pub(crate) fn', 'fn')
            
    with open(filepath, 'w') as f:
        f.writelines(lines)

remove_pub_crate_in_traits('../cvkg-render-gpu/src/api.rs')
remove_pub_crate_in_traits('../cvkg-render-gpu/src/renderer/mod.rs')
