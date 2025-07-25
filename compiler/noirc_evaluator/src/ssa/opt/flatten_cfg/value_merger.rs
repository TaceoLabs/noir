use acvm::{FieldElement, acir::AcirField};
use fxhash::FxHashMap as HashMap;
use noirc_errors::call_stack::CallStackId;

use crate::ssa::ir::{
    basic_block::BasicBlockId,
    dfg::DataFlowGraph,
    instruction::{BinaryOp, Instruction},
    types::{NumericType, Type},
    value::ValueId,
};

pub(crate) struct ValueMerger<'a> {
    dfg: &'a mut DataFlowGraph,
    block: BasicBlockId,

    // Maps SSA array values with a slice type to their size.
    // This must be computed before merging values.
    slice_sizes: &'a mut HashMap<ValueId, u32>,

    call_stack: CallStackId,
}

impl<'a> ValueMerger<'a> {
    pub(crate) fn new(
        dfg: &'a mut DataFlowGraph,
        block: BasicBlockId,
        slice_sizes: &'a mut HashMap<ValueId, u32>,
        call_stack: CallStackId,
    ) -> Self {
        ValueMerger { dfg, block, slice_sizes, call_stack }
    }

    /// Merge two values a and b from separate basic blocks to a single value.
    /// If these two values are numeric, the result will be
    /// `then_condition * (then_value - else_value) + else_value`.
    /// Otherwise, if the values being merged are arrays, a new array will be made
    /// recursively from combining each element of both input arrays.
    ///
    /// It is currently an error to call this function on reference or function values
    /// as it is less clear how to merge these.
    pub(crate) fn merge_values(
        &mut self,
        then_condition: ValueId,
        else_condition: ValueId,
        then_value: ValueId,
        else_value: ValueId,
    ) -> ValueId {
        if then_value == else_value {
            return then_value;
        }

        match self.dfg.type_of_value(then_value) {
            Type::Numeric(_) => Self::merge_numeric_values(
                self.dfg,
                self.block,
                then_condition,
                else_condition,
                then_value,
                else_value,
            ),
            typ @ Type::Array(_, _) => {
                self.merge_array_values(typ, then_condition, else_condition, then_value, else_value)
            }
            typ @ Type::Slice(_) => {
                self.merge_slice_values(typ, then_condition, else_condition, then_value, else_value)
            }
            Type::Reference(_) => panic!("Cannot return references from an if expression"),
            Type::Function => panic!("Cannot return functions from an if expression"),
        }
    }

    /// Merge two numeric values a and b from separate basic blocks to a single value. This
    /// function would return the result of `if c { a } else { b }` as  `c*a + (!c)*b`.
    pub(crate) fn merge_numeric_values(
        dfg: &mut DataFlowGraph,
        block: BasicBlockId,
        then_condition: ValueId,
        else_condition: ValueId,
        then_value: ValueId,
        else_value: ValueId,
    ) -> ValueId {
        let then_type = dfg.type_of_value(then_value).unwrap_numeric();
        let else_type = dfg.type_of_value(else_value).unwrap_numeric();
        assert_eq!(
            then_type, else_type,
            "Expected values merged to be of the same type but found {then_type} and {else_type}"
        );

        if then_value == else_value {
            return then_value;
        }

        let then_call_stack = dfg.get_value_call_stack_id(then_value);
        let else_call_stack = dfg.get_value_call_stack_id(else_value);

        let call_stack = if then_call_stack.is_root() { else_call_stack } else { then_call_stack };

        // We must cast the bool conditions to the actual numeric type used by each value.
        let cast = Instruction::Cast(then_condition, then_type);
        let then_condition =
            dfg.insert_instruction_and_results(cast, block, None, call_stack).first();

        let cast = Instruction::Cast(else_condition, else_type);
        let else_condition =
            dfg.insert_instruction_and_results(cast, block, None, call_stack).first();

        // Unchecked mul because `then_condition` will be 1 or 0
        let mul =
            Instruction::binary(BinaryOp::Mul { unchecked: true }, then_condition, then_value);
        let then_value = dfg.insert_instruction_and_results(mul, block, None, call_stack).first();

        // Unchecked mul because `else_condition` will be 1 or 0
        let mul =
            Instruction::binary(BinaryOp::Mul { unchecked: true }, else_condition, else_value);
        let else_value = dfg.insert_instruction_and_results(mul, block, None, call_stack).first();

        // Unchecked add because one of the values will always be 0
        let add = Instruction::binary(BinaryOp::Add { unchecked: true }, then_value, else_value);
        dfg.insert_instruction_and_results(add, block, None, call_stack).first()
    }

    /// Given an if expression that returns an array: `if c { array1 } else { array2 }`,
    /// this function will recursively merge array1 and array2 into a single resulting array
    /// by creating a new array containing the result of self.merge_values for each element.
    pub(crate) fn merge_array_values(
        &mut self,
        typ: Type,
        then_condition: ValueId,
        else_condition: ValueId,
        then_value: ValueId,
        else_value: ValueId,
    ) -> ValueId {
        let mut merged = im::Vector::new();

        let (element_types, len) = match &typ {
            Type::Array(elements, len) => (elements, *len),
            _ => panic!("Expected array type"),
        };

        for i in 0..len {
            for (element_index, element_type) in element_types.iter().enumerate() {
                let index =
                    ((i * element_types.len() as u32 + element_index as u32) as u128).into();
                let index = self.dfg.make_constant(index, NumericType::length_type());

                let typevars = Some(vec![element_type.clone()]);

                let mut get_element = |array, typevars| {
                    let get = Instruction::ArrayGet { array, index };
                    self.dfg
                        .insert_instruction_and_results(get, self.block, typevars, self.call_stack)
                        .first()
                };

                let then_element = get_element(then_value, typevars.clone());
                let else_element = get_element(else_value, typevars);

                merged.push_back(self.merge_values(
                    then_condition,
                    else_condition,
                    then_element,
                    else_element,
                ));
            }
        }

        let instruction = Instruction::MakeArray { elements: merged, typ };
        self.dfg
            .insert_instruction_and_results(instruction, self.block, None, self.call_stack)
            .first()
    }

    fn merge_slice_values(
        &mut self,
        typ: Type,
        then_condition: ValueId,
        else_condition: ValueId,
        then_value_id: ValueId,
        else_value_id: ValueId,
    ) -> ValueId {
        let mut merged = im::Vector::new();

        let element_types = match &typ {
            Type::Slice(elements) => elements,
            _ => panic!("Expected slice type"),
        };

        let then_len = self.slice_sizes.get(&then_value_id).copied().unwrap_or_else(|| {
            let (slice, typ) = self.dfg.get_array_constant(then_value_id).unwrap_or_else(|| {
                panic!("ICE: Merging values during flattening encountered slice {then_value_id} without a preset size");
            });
            (slice.len() / typ.element_types().len()) as u32
        });

        let else_len = self.slice_sizes.get(&else_value_id).copied().unwrap_or_else(|| {
            let (slice, typ) = self.dfg.get_array_constant(else_value_id).unwrap_or_else(|| {
                panic!("ICE: Merging values during flattening encountered slice {else_value_id} without a preset size");
            });
            (slice.len() / typ.element_types().len()) as u32
        });

        let len = then_len.max(else_len);

        for i in 0..len {
            for (element_index, element_type) in element_types.iter().enumerate() {
                let index_u32 = i * element_types.len() as u32 + element_index as u32;
                let index_value = (index_u32 as u128).into();
                let index = self.dfg.make_constant(index_value, NumericType::NativeField);

                let typevars = Some(vec![element_type.clone()]);

                let mut get_element = |array, typevars, len| {
                    // The smaller slice is filled with placeholder data. Codegen for slice accesses must
                    // include checks against the dynamic slice length so that this placeholder data is not incorrectly accessed.
                    if len <= index_u32 {
                        self.make_slice_dummy_data(element_type)
                    } else {
                        let get = Instruction::ArrayGet { array, index };
                        self.dfg
                            .insert_instruction_and_results(
                                get,
                                self.block,
                                typevars,
                                self.call_stack,
                            )
                            .first()
                    }
                };

                let then_element = get_element(
                    then_value_id,
                    typevars.clone(),
                    then_len * element_types.len() as u32,
                );
                let else_element =
                    get_element(else_value_id, typevars, else_len * element_types.len() as u32);

                merged.push_back(self.merge_values(
                    then_condition,
                    else_condition,
                    then_element,
                    else_element,
                ));
            }
        }

        let instruction = Instruction::MakeArray { elements: merged, typ };
        let call_stack = self.call_stack;
        self.dfg.insert_instruction_and_results(instruction, self.block, None, call_stack).first()
    }

    /// Construct a dummy value to be attached to the smaller of two slices being merged.
    /// We need to make sure we follow the internal element type structure of the slice type
    /// even for dummy data to ensure that we do not have errors later in the compiler,
    /// such as with dynamic indexing of non-homogenous slices.
    fn make_slice_dummy_data(&mut self, typ: &Type) -> ValueId {
        match typ {
            Type::Numeric(numeric_type) => {
                let zero = FieldElement::zero();
                self.dfg.make_constant(zero, *numeric_type)
            }
            Type::Array(element_types, len) => {
                let mut array = im::Vector::new();
                for _ in 0..*len {
                    for typ in element_types.iter() {
                        array.push_back(self.make_slice_dummy_data(typ));
                    }
                }
                let instruction = Instruction::MakeArray { elements: array, typ: typ.clone() };
                let call_stack = self.call_stack;
                self.dfg
                    .insert_instruction_and_results(instruction, self.block, None, call_stack)
                    .first()
            }
            Type::Slice(_) => {
                // TODO(#3188): Need to update flattening to use true user facing length of slices
                // to accurately construct dummy data
                unreachable!("ICE: Cannot return a slice of slices from an if expression")
            }
            Type::Reference(_) => {
                unreachable!("ICE: Merging references is unsupported")
            }
            Type::Function => {
                unreachable!("ICE: Merging functions is unsupported")
            }
        }
    }
}
