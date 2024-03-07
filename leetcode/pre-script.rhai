let arg_names = [];
let arg_types = []; 

arg_names += variable::prompt("Argument label", "");
arg_names[0].trim();

while arg_names[-1] != "" {
    arg_types += variable::prompt("Type of `" + arg_names[-1] + "`");
    arg_types[-1].trim();
    arg_names += variable::prompt("Argument label", "");
    arg_names[-1].trim();
}
arg_names.pop();

let zipped = arg_names.zip(arg_types, |a, b| [a, b]); 

let signature = "(";
for values in zipped {
    signature += values[0] + ": " + values[1] + ", ";
}
if signature.ends_with(", ") {
    signature.pop(2);
}
signature += ")";

let return_type = variable::prompt("Return type", "");
return_type.trim();
if return_type != "" {
    signature += " -> " + return_type;
}
variable::set("signature", signature);

if !(arg_names.is_empty() || return_type == "") {
    let example_prompt = variable::prompt("Would you like to add examples?", false);
    let examples = "";
    let example_number = 1;
    while example_prompt {
        let example = "example!(" + example_number + ", ";
        example_number += 1;

        let args = "";
        for values in zipped {
            let value = variable::prompt("Value of `" + values[0] + ": " + values[1] + "`");
            value.trim();
            value.replace("[", "vec![");
            args += value + ", ";
        }
        args.pop(2);
        args += ");\n\t";

        let value = variable::prompt("Expected value (`" + return_type + "`)");
        value.trim();
        value.replace("[", "vec![");

        examples += example + value + ", " + args;
        example_prompt = variable::prompt("Add another example?", false);
    }

    if examples != "" {
        variable::set("examples", examples);
    }
}
