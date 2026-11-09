use super::*;

impl ParseContext<'_> {
    pub(crate) fn condition_value(&mut self) -> Result<wir::ValueId> {
        self.value_with_condition(true)
    }

    pub(crate) fn value(&mut self) -> Result<wir::ValueId> {
        self.value_with_condition(false)
    }

    fn value_with_condition(&mut self, condition: bool) -> Result<wir::ValueId> {
        let saved = std::mem::replace(&mut self.in_condition, condition);
        let result = self.value_inner();
        self.in_condition = saved;
        result
    }

    pub(crate) fn with_call<T>(
        &mut self,
        call_id: &str,
        f: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        self.call_stack.push(call_id.to_string());
        let result = f(self);
        self.call_stack.pop();
        result
    }
}
